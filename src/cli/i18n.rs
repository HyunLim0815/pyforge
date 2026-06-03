//! `pyforge i18n` 子命令实现。
//!
//! - `i18n list`：列出所有可用语言（内置 + 已安装）
//! - `i18n install <lang>`：从社区仓库下载语言包

use super::Cli;
use crate::i18n::loader;
use crate::output::json;
use crate::t;
use clap::{Args, Subcommand};
use std::fs;

/// 社区语言包仓库基础 URL。可通过 `PYFORGE_I18N_REPO` 环境变量覆盖。
const DEFAULT_REPO_URL: &str = "https://raw.githubusercontent.com/pyforge/i18n-packs/main";

fn repo_url() -> String {
    std::env::var("PYFORGE_I18N_REPO").unwrap_or_else(|_| DEFAULT_REPO_URL.to_string())
}

#[derive(Args, Debug)]
pub struct I18nArgs {
    #[command(subcommand)]
    pub command: I18nCommand,
}

#[derive(Subcommand, Debug)]
pub enum I18nCommand {
    /// 列出所有可用语言
    #[command(name = "list")]
    List,
    /// 从社区仓库安装语言包
    #[command(name = "install")]
    Install(InstallArgs),
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// 语言代码（如 ja, ko, fr）
    pub lang: String,
}

#[derive(serde::Serialize, Clone, Debug)]
struct LangListEntry {
    code: String,
    language: String,
    source: String,
    key_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

pub fn handle(cli: &Cli, args: &I18nArgs) -> Result<(), i32> {
    match &args.command {
        I18nCommand::List => handle_list(cli),
        I18nCommand::Install(a) => handle_install(cli, &a.lang),
    }
}

fn handle_list(cli: &Cli) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::i18n::list").entered();

    let mut entries: Vec<LangListEntry> = Vec::new();

    // 内置语言
    for (code, name) in loader::BUILTIN_LANGS {
        entries.push(LangListEntry {
            code: code.to_string(),
            language: name.to_string(),
            source: t!("i18n.list.builtin"),
            key_count: 0, // 内置语言不统计 key 数
            version: None,
        });
    }

    // 外部语言
    let external = crate::i18n::list_external_langs();
    for lang in &external {
        entries.push(LangListEntry {
            code: lang.code.clone(),
            language: lang.language.clone(),
            source: t!("i18n.list.external"),
            key_count: lang.key_count,
            version: lang.version.clone(),
        });
    }

    if cli.json {
        println!("{}", json::success(serde_json::json!({ "languages": &entries })));
    } else {
        eprintln!("{}", t!("i18n.list.header"));
        for e in &entries {
            println!("  {:<8} {:<16} {} ({} keys)", e.code, e.language, e.source, e.key_count);
        }
        if external.is_empty() {
            eprintln!("{}", t!("i18n.list.none"));
        }
    }

    Ok(())
}

/// 校验语言代码只包含安全字符，防止路径遍历和 URL 注入。
fn validate_lang_code(lang: &str) -> Result<(), &'static str> {
    if lang.is_empty() || lang.contains('.') || lang.contains('/') || lang.contains('\\') || lang.contains(':') || lang.contains('?') || lang.contains('#') {
        return Err("invalid language code");
    }
    if !lang.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err("invalid language code");
    }
    Ok(())
}

fn handle_install(cli: &Cli, lang: &str) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::i18n::install").entered();

    if let Err(e) = validate_lang_code(lang) {
        let msg = format!("{}: {}", e, lang);
        if cli.json {
            println!("{}", json::error("I18N", "INVALID_LANG_CODE", &msg));
        }
        eprintln!("{}", msg);
        return Err(1);
    }

    let dir = loader::lang_dir().ok_or_else(|| {
        eprintln!("{}", t!("i18n.install.fail", "系统数据目录不可用"));
        2
    })?;
    fs::create_dir_all(&dir).map_err(|e| {
        eprintln!("{}", t!("i18n.install.fail", e));
        2
    })?;

    let dest = dir.join(format!("{}.json", lang));
    if dest.exists() {
        if cli.json {
            println!(
                "{}",
                json::error("I18N", "ALREADY_EXISTS", &t!("i18n.install.already", lang))
            );
        } else {
            eprintln!("{}", t!("i18n.install.already", lang));
        }
        return Err(1);
    }

    // 下载
    let url = format!("{}/{}.json", repo_url(), lang);
    let body = match download(&url) {
        Ok(b) => b,
        Err(e) => {
            if cli.json {
                println!(
                    "{}",
                    json::error("I18N", "DOWNLOAD_FAILED", &t!("i18n.install.fail", &e))
                );
            } else {
                eprintln!("{}", t!("i18n.install.fail", e));
            }
            return Err(2);
        }
    };

    // schema 校验（先写临时文件再验证）
    let tmp = dest.with_extension("tmp");
    fs::write(&tmp, &body).map_err(|e| {
        eprintln!("{}", t!("i18n.install.fail", e));
        2
    })?;

    match loader::load_lang_file(&tmp) {
        Ok(_) => {
            fs::rename(&tmp, &dest).map_err(|e| {
                eprintln!("{}", t!("i18n.install.fail", e));
                2
            })?;
            if cli.json {
                println!(
                    "{}",
                    json::success(serde_json::json!({
                        "lang": lang,
                        "path": dest.display().to_string()
                    }))
                );
            } else {
                eprintln!("{}", t!("i18n.install.success", lang));
            }
            Ok(())
        }
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            if cli.json {
                println!(
                    "{}",
                    json::error("I18N", "INVALID_FORMAT", &t!("i18n.install.invalid"))
                );
            } else {
                eprintln!("{}: {}", t!("i18n.install.invalid"), e);
            }
            Err(1)
        }
    }
}

fn download(url: &str) -> Result<String, String> {
    // 支持 mock：PYFORGE_I18N_MOCK=<json_content>
    if let Ok(mock) = std::env::var("PYFORGE_I18N_MOCK") {
        return Ok(mock);
    }

    let mut resp = ureq::get(url)
        .call()
        .map_err(|e| format!("HTTP 请求失败: {}", e))?;

    resp.body_mut()
        .read_to_string()
        .map_err(|e| format!("读取响应失败: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_mock_returns_content() {
        std::env::set_var("PYFORGE_I18N_MOCK", r#"{"meta":{"language":"test"},"strings":{"k":"v"}}"#);
        let result = download("http://unused");
        std::env::remove_var("PYFORGE_I18N_MOCK");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test"));
    }

    #[test]
    fn repo_url_default() {
        std::env::remove_var("PYFORGE_I18N_REPO");
        let url = repo_url();
        assert!(url.contains("pyforge"));
    }

    #[test]
    fn repo_url_from_env() {
        std::env::set_var("PYFORGE_I18N_REPO", "http://localhost:8080");
        let url = repo_url();
        assert_eq!(url, "http://localhost:8080");
        std::env::remove_var("PYFORGE_I18N_REPO");
    }
}
