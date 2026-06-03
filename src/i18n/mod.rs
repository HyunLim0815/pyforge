//! i18n 消息模块。
//!
//! 所有用户可见字符串通过 `t!()` 宏输出，集中管理，禁止内联。
//! [Source: architecture.md §i18n Message Pattern]

mod en;
pub mod loader;
mod zh;

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

/// 0 = 中文, 1 = 英文, 2 = 外部语言
static LANG: AtomicU8 = AtomicU8::new(0);

/// 外部语言包缓存。
static EXTERNAL_LANGS: Mutex<Vec<(String, loader::LangPack)>> = Mutex::new(Vec::new());

/// 当前语言代码（zh/en/外部代码）。
static LANG_CODE: Mutex<String> = Mutex::new(String::new());

/// 安全获取锁，恢复中毒锁而非 panic。
fn safe_lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

/// 设置全局语言。
pub fn set_lang(lang: &str) {
    let lower = lang.to_lowercase();
    let v = match lower.as_str() {
        "en" | "en-us" | "en_us" => 1,
        "zh" | "zh-cn" | "zh_cn" => 0,
        _ => {
            // 检查是否为已加载的外部语言
            let langs = safe_lock(&EXTERNAL_LANGS);
            if langs.iter().any(|(c, _)| c == &lower) {
                2
            } else {
                0 // 未知语言回退中文
            }
        }
    };
    LANG.store(v, Ordering::Release);
    let mut code = safe_lock(&LANG_CODE);
    *code = lang.to_lowercase();
}

/// 获取当前语言（0=中文 1=英文 2=外部），供宏和测试使用。
#[allow(dead_code)]
pub fn current_lang() -> u8 {
    LANG.load(Ordering::Acquire)
}

/// 获取当前语言代码。
#[allow(dead_code)]
pub fn current_lang_code() -> String {
    safe_lock(&LANG_CODE).clone()
}

/// 加载外部语言包。
pub fn init_external() {
    let langs = loader::load_external_langs();
    let mut cache = safe_lock(&EXTERNAL_LANGS);
    *cache = langs;
}

/// 获取已加载的外部语言信息列表。
pub fn list_external_langs() -> Vec<loader::LangInfo> {
    let langs = safe_lock(&EXTERNAL_LANGS);
    langs
        .iter()
        .map(|(code, pack)| loader::LangInfo {
            code: code.clone(),
            language: pack.meta.language.clone(),
            contributors: pack.meta.contributors.clone(),
            version: pack.meta.version.clone(),
            key_count: pack.strings.len(),
            source: "external".into(),
        })
        .collect()
}

/// i18n 消息宏。
///
/// - `t!("key")` 输出当前语言的 key 对应文本
/// - `t!("key", arg1, arg2)` 用参数替换占位符 `{0}`, `{1}` 等
#[macro_export]
macro_rules! t {
    ($key:expr) => {{
        $crate::i18n::msg($key, &[])
    }};
    ($key:expr, $($arg:expr),+ $(,)?) => {{
        $crate::i18n::msg($key, &[$($arg.to_string()),+])
    }};
}

/// 内部消息解析函数：查找 key，替换占位符，缺失时回退。
pub fn msg(key: &str, args: &[String]) -> String {
    let template = lookup(key);
    if template == key {
        tracing::warn!("i18n key not found: {}", key);
        return key.to_string();
    }
    let mut result = template;
    for (i, arg) in args.iter().enumerate() {
        result = result.replace(&format!("{{{}}}", i), arg);
    }
    // 移除未替换的占位符，避免残留 {N} 在输出中
    for i in args.len()..args.len() + 10 {
        let placeholder = format!("{{{}}}", i);
        if result.contains(&placeholder) {
            tracing::warn!("i18n key '{}' missing arg {}", key, i);
            result = result.replace(&placeholder, "");
        } else {
            break;
        }
    }
    result
}

/// 根据当前语言查找消息模板。
///
/// 回退顺序：外部语言 → 内置语言（zh/en）→ key 字符串
fn lookup(key: &str) -> String {
    let lang = LANG.load(Ordering::Acquire);

    // 外部语言
    if lang == 2 {
        let code = safe_lock(&LANG_CODE).clone();
        let langs = safe_lock(&EXTERNAL_LANGS);
        if let Some(val) = loader::lookup_external(&langs, &code, key) {
            return val;
        }
        // 外部语言缺失 key → 回退内置中文
        return zh::get(key).unwrap_or_else(|| key.to_string());
    }

    if lang == 1 {
        en::get(key).unwrap_or_else(|| key.to_string())
    } else {
        zh::get(key).unwrap_or_else(|| key.to_string())
    }
}

/// 初始化语言：`--lang` > `PYFORGE_LANG` > 默认中文。
///
/// 支持内置语言（zh/en）和外部语言包代码。
/// 当 `--lang` 非空但无效时，直接回退中文，不再检查环境变量。
pub fn init_lang(cli_lang: &str) {
    let effective = if cli_lang == "en" {
        "en".to_string()
    } else if cli_lang == "zh" {
        "zh".to_string()
    } else if !cli_lang.is_empty() {
        // --lang 非空：检查是否为已加载的外部语言，否则直接回退中文
        let langs = safe_lock(&EXTERNAL_LANGS);
        if langs.iter().any(|(c, _)| c == cli_lang) {
            cli_lang.to_string()
        } else {
            "zh".to_string()
        }
    } else {
        // --lang 未传，检查环境变量
        match std::env::var("PYFORGE_LANG") {
            Ok(v) if v.to_lowercase() == "en" => "en".to_string(),
            Ok(v) if !v.is_empty() => {
                let lower = v.to_lowercase();
                let langs = safe_lock(&EXTERNAL_LANGS);
                if langs.iter().any(|(c, _)| c == &lower) {
                    lower
                } else {
                    "zh".to_string()
                }
            }
            _ => "zh".to_string(),
        }
    };
    set_lang(&effective);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn lang_defaults_to_zh() {
        let _g = TEST_LOCK.lock().unwrap();
        set_lang("zh");
        assert_eq!(t!("test.hello"), "你好");
    }

    #[test]
    fn lang_switch_to_en() {
        let _g = TEST_LOCK.lock().unwrap();
        set_lang("en");
        assert_eq!(t!("test.hello"), "Hello");
    }

    #[test]
    fn missing_key_returns_key() {
        let _g = TEST_LOCK.lock().unwrap();
        set_lang("zh");
        assert_eq!(t!("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn macro_with_args() {
        let _g = TEST_LOCK.lock().unwrap();
        set_lang("zh");
        assert_eq!(t!("track.success", "my-api"), "已注册项目: my-api");
        set_lang("en");
        assert_eq!(t!("track.success", "my-api"), "Project registered: my-api");
    }

    #[test]
    fn external_lang_lookup() {
        let _g = TEST_LOCK.lock().unwrap();

        let pack = loader::LangPack {
            meta: loader::LangMeta {
                language: "日本語".into(),
                contributors: vec![],
                version: None,
            },
            strings: {
                let mut m = std::collections::HashMap::new();
                m.insert("test.hello".into(), "こんにちは".into());
                m
            },
        };
        if let Ok(mut langs) = EXTERNAL_LANGS.lock() {
            *langs = vec![("ja".into(), pack)];
        }

        set_lang("ja");
        assert_eq!(t!("test.hello"), "こんにちは");
    }

    #[test]
    fn external_lang_fallback_to_zh() {
        let _g = TEST_LOCK.lock().unwrap();

        let pack = loader::LangPack {
            meta: loader::LangMeta {
                language: "日本語".into(),
                contributors: vec![],
                version: None,
            },
            strings: std::collections::HashMap::new(),
        };
        if let Ok(mut langs) = EXTERNAL_LANGS.lock() {
            *langs = vec![("ja".into(), pack)];
        }

        set_lang("ja");
        // "test.hello" 不在日语包中，应回退到中文
        assert_eq!(t!("test.hello"), "你好");

        // 清理
        if let Ok(mut langs) = EXTERNAL_LANGS.lock() {
            *langs = vec![];
        }
        set_lang("zh");
    }
}
