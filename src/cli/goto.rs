//! `pyforge goto` 命令实现。
//!
//! Plain 模式 stdout 必须只输出项目路径，便于 `cd $(pyforge goto name)`。

use super::Cli;
use crate::index::store::{IndexStore, JsonStore};
use crate::output::json;
use crate::t;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct GotoArgs {
    /// 项目名称
    pub name: String,
}

pub fn handle(cli: &Cli, args: &GotoArgs) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::goto").entered();
    let index_path = std::env::var("PYFORGE_INDEX_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| JsonStore::default_path());
    let store = JsonStore::open(index_path).map_err(|e| {
        eprintln!("{}", e);
        2
    })?;

    let Some(project) = store.get_project(&args.name) else {
        let msg = t!("info.not_found", &args.name);
        if cli.json {
            println!("{}", json::error("INDEX", "INDEX_PROJECT_NOT_FOUND", &msg));
        }
        eprintln!("{}", msg);
        return Err(1);
    };

    if cli.json {
        println!("{}", json::success(serde_json::json!({ "path": project.path })));
    } else {
        println!("{}", project.path);
    }
    Ok(())
}
