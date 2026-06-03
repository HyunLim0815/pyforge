//! 外部语言包加载器。
//!
//! 扫描 `~/.pyforge/i18n/*.json`，校验 schema，返回可用语言列表。

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 外部语言包元数据。
#[derive(Debug, Clone, Deserialize)]
pub struct LangMeta {
    pub language: String,
    #[serde(default)]
    pub contributors: Vec<String>,
    #[serde(default)]
    pub version: Option<String>,
}

/// 外部语言包完整结构。
#[derive(Debug, Clone, Deserialize)]
pub struct LangPack {
    pub meta: LangMeta,
    pub strings: HashMap<String, String>,
}

/// 已加载的语言包信息（用于 list 展示）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct LangInfo {
    pub code: String,
    pub language: String,
    pub contributors: Vec<String>,
    pub version: Option<String>,
    pub key_count: usize,
    pub source: String, // "builtin" 或 "external"
}

/// 内置语言列表。
pub const BUILTIN_LANGS: &[(&str, &str)] = &[("zh", "简体中文"), ("en", "English")];

/// 获取外部语言包目录路径。不可用时返回 None。
pub fn lang_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("pyforge").join("i18n"))
}

/// 扫描并加载所有外部语言包。
///
/// 返回 `(code, LangPack)` 列表。无效文件跳过并 warn。
pub fn load_external_langs() -> Vec<(String, LangPack)> {
    let dir = match lang_dir() {
        Some(d) => d,
        None => return Vec::new(),
    };
    if !dir.exists() {
        return Vec::new();
    }

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut result = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.extension().is_some_and(|e| e == "json") {
            continue;
        }
        let code = match path.file_stem().and_then(|s| s.to_str()) {
            Some(c) => c.to_string(),
            None => continue,
        };

        match load_lang_file(&path) {
            Ok(pack) => {
                result.push((code, pack));
            }
            Err(e) => {
                tracing::warn!("跳过无效语言包 {}: {}", path.display(), e);
            }
        }
    }

    result
}

/// 从文件加载并校验语言包。
pub fn load_lang_file(path: &std::path::Path) -> Result<LangPack, String> {
    let mut content = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    // 去除 UTF-8 BOM（Windows 记事本保存的文件常带 BOM）
    if content.starts_with('\u{FEFF}') {
        content = content[3..].to_string();
    }
    let pack: LangPack =
        serde_json::from_str(&content).map_err(|e| format!("JSON 解析失败: {}", e))?;

    // schema 校验
    if pack.meta.language.is_empty() {
        return Err("meta.language 不能为空".into());
    }
    if pack.strings.is_empty() {
        return Err("strings 不能为空".into());
    }

    Ok(pack)
}

/// 从外部语言包中查找消息。
pub fn lookup_external(langs: &[(String, LangPack)], code: &str, key: &str) -> Option<String> {
    langs
        .iter()
        .find(|(c, _)| c == code)
        .and_then(|(_, pack)| pack.strings.get(key).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_valid_lang_pack() {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("ja.json");
        fs::write(
            &file,
            r#"{
                "meta": {"language": "日本語", "contributors": ["test"], "version": "1.0"},
                "strings": {"test.hello": "こんにちは"}
            }"#,
        )
        .unwrap();

        let pack = load_lang_file(&file).unwrap();
        assert_eq!(pack.meta.language, "日本語");
        assert_eq!(pack.strings.get("test.hello").unwrap(), "こんにちは");
    }

    #[test]
    fn reject_missing_language() {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("bad.json");
        fs::write(
            &file,
            r#"{
                "meta": {"language": "", "contributors": []},
                "strings": {"key": "val"}
            }"#,
        )
        .unwrap();

        let err = load_lang_file(&file).unwrap_err();
        assert!(err.contains("meta.language"));
    }

    #[test]
    fn reject_missing_strings() {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("bad.json");
        fs::write(
            &file,
            r#"{
                "meta": {"language": "Test", "contributors": []},
                "strings": {}
            }"#,
        )
        .unwrap();

        let err = load_lang_file(&file).unwrap_err();
        assert!(err.contains("strings"));
    }

    #[test]
    fn reject_invalid_json() {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("bad.json");
        fs::write(&file, "not json").unwrap();

        let err = load_lang_file(&file).unwrap_err();
        assert!(err.contains("JSON"));
    }

    #[test]
    fn load_external_langs_from_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        let i18n_dir = dir.path().join("pyforge").join("i18n");
        fs::create_dir_all(&i18n_dir).unwrap();

        fs::write(
            i18n_dir.join("ja.json"),
            r#"{"meta":{"language":"日本語"},"strings":{"k":"v"}}"#,
        )
        .unwrap();
        fs::write(
            i18n_dir.join("ko.json"),
            r#"{"meta":{"language":"한국어"},"strings":{"k":"v"}}"#,
        )
        .unwrap();
        // 非 JSON 文件应被忽略
        fs::write(i18n_dir.join("readme.txt"), "ignored").unwrap();

        // 临时覆盖 lang_dir
        // 由于 lang_dir() 使用 dirs::data_dir()，这里直接测试 load_lang_file
        let ja = load_lang_file(&i18n_dir.join("ja.json")).unwrap();
        assert_eq!(ja.meta.language, "日本語");
        let ko = load_lang_file(&i18n_dir.join("ko.json")).unwrap();
        assert_eq!(ko.meta.language, "한국어");
    }

    #[test]
    fn lookup_external_finds_key() {
        let pack = LangPack {
            meta: LangMeta {
                language: "日本語".into(),
                contributors: vec![],
                version: None,
            },
            strings: {
                let mut m = HashMap::new();
                m.insert("test.hello".into(), "こんにちは".into());
                m
            },
        };
        let langs = vec![("ja".into(), pack)];

        assert_eq!(
            lookup_external(&langs, "ja", "test.hello"),
            Some("こんにちは".into())
        );
        assert_eq!(lookup_external(&langs, "ja", "missing"), None);
        assert_eq!(lookup_external(&langs, "ko", "test.hello"), None);
    }
}
