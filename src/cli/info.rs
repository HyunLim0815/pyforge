//! `pyforge info` 命令实现。

use super::Cli;
use crate::index::store::{IndexStore, JsonStore};
use crate::output::json;
use crate::t;
use clap::Args;
use serde_json::json as json_value;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// 项目名称
    pub name: String,
}

pub fn handle(cli: &Cli, args: &InfoArgs) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::info").entered();
    let store = open_store()?;

    let Some(project) = store.get_project(&args.name) else {
        let msg = t!("info.not_found", &args.name);
        if cli.json {
            println!("{}", json::error("INDEX", "INDEX_PROJECT_NOT_FOUND", &msg));
        }
        eprintln!("{}", msg);
        return Err(1);
    };

    if cli.json {
        println!("{}", json::success(json_value!({ "project": project })));
    } else {
        println!("name: {}", project.name);
        println!("path: {}", project.path);
        println!(
            "python_version: {}",
            project.python_version.unwrap_or_else(|| "unknown".into())
        );
        println!(
            "git_status: {}",
            project.git_status.unwrap_or_else(|| "unknown".into())
        );
        println!(
            "last_modified: {}",
            project.last_modified.unwrap_or_else(|| "unknown".into())
        );
        println!(
            "deps_count: {}",
            project
                .deps_count
                .map(|n| n.to_string())
                .unwrap_or_else(|| "unknown".into())
        );
    }
    Ok(())
}

fn open_store() -> Result<JsonStore, i32> {
    let index_path = std::env::var("PYFORGE_INDEX_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| JsonStore::default_path());
    JsonStore::open(index_path).map_err(|e| {
        eprintln!("{}", e);
        2
    })
}
