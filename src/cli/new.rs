//! `pyforge new` 命令实现。

use super::Cli;
use crate::index::detector;
use crate::index::store::{IndexStore, JsonStore};
use crate::index::types::ProjectInfo;
use crate::output::json;
use crate::t;
use crate::templates;
use crate::uv::{bridge, error::UvError};
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct NewArgs {
    /// 项目名称
    pub name: String,

    /// 模板类型（Story 2.2 实现）
    #[arg(long)]
    pub template: Option<String>,

    /// 创建但不注册到索引
    #[arg(long)]
    pub no_track: bool,
}

/// 校验项目名不包含路径遍历组件。
fn validate_project_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty()
        || name.contains("..")
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
    {
        return Err("invalid project name");
    }
    Ok(())
}

pub fn handle(cli: &Cli, args: &NewArgs) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::new").entered();

    if let Err(e) = validate_project_name(&args.name) {
        let msg = format!("{}: {}", e, args.name);
        if cli.json {
            println!("{}", json::error("CLI", "CLI_INVALID_ARG", &msg));
        }
        eprintln!("{}", msg);
        return Err(1);
    }

    let cwd = std::env::current_dir().map_err(|e| {
        eprintln!("{}", e);
        2
    })?;
    let project_path = cwd.join(&args.name);

    if project_path.exists() {
        let msg = t!("new.dir_exists", &args.name);
        if cli.json {
            println!("{}", json::error("CLI", "CLI_INVALID_ARG", &msg));
        }
        eprintln!("{}", msg);
        return Err(1);
    }

    if let Err(e) = bridge::init(&args.name, &cwd) {
        let (code, namespace, err_code, msg) = match e {
            UvError::NotInstalled => (1, "UV", "UV_NOT_INSTALLED", t!("new.uv_missing")),
            UvError::InitFailed(msg) => (1, "UV", "UV_INIT_FAILED", msg),
            UvError::ParseError(msg) => (2, "UV", "UV_INIT_FAILED", msg),
            UvError::Io(e) => (2, "UV", "UV_INIT_FAILED", e.to_string()),
        };
        if cli.json {
            println!("{}", json::error(namespace, err_code, &msg));
        }
        eprintln!("{}", msg);
        return Err(code);
    }

    // 模板渲染（在 uv init 之后展开，避免被 uv 覆盖）
    if let Some(ref tpl) = args.template {
        if let Err(e) = templates::render(tpl, &args.name, &project_path) {
            let msg = match &e {
                templates::TemplateError::UnknownTemplate(_) => {
                    let available = templates::BUILTIN_TEMPLATES.join(", ");
                    t!("template.unknown", tpl, &available)
                }
                templates::TemplateError::Io(e) => e.to_string(),
            };
            if cli.json {
                println!("{}", json::error("CLI", "CLI_INVALID_ARG", &msg));
            }
            eprintln!("{}", msg);
            return Err(1);
        }
    }

    let canonical = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.clone());
    let mut project_json = None;

    if !args.no_track {
        let detection = detector::detect(&canonical);
        let info = ProjectInfo {
            name: args.name.clone(),
            path: canonical.display().to_string(),
            python_version: detection.python_version,
            toolchain: detection.toolchain,
            git_remote: None,
            git_branch: None,
            git_status: None,
            deps_count: None,
            created_at: None,
            last_modified: None,
            tags: vec![],
            description: None,
        };
        let index_path = std::env::var("PYFORGE_INDEX_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| JsonStore::default_path());
        let mut store = JsonStore::open(index_path).map_err(|e| {
            eprintln!("{}", e);
            2
        })?;
        store.add_project(info.clone()).map_err(|e| {
            eprintln!("{}", e);
            2
        })?;
        project_json = Some(info);
    }

    let msg = t!("new.success", &args.name);
    if cli.json {
        println!(
            "{}",
            json::success(serde_json::json!({
                "name": args.name,
                "path": canonical,
                "tracked": !args.no_track,
                "project": project_json,
            }))
        );
    }
    eprintln!("{}", msg);
    Ok(())
}
