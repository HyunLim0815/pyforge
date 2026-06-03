//! `pyforge untrack` 命令实现。

use super::Cli;
use crate::index::error::IndexError;
use crate::index::store::{IndexStore, JsonStore};
use crate::output::json;
use crate::t;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct UntrackArgs {
    /// 项目名称
    pub name: String,
}

pub fn handle(cli: &Cli, args: &UntrackArgs) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::untrack").entered();
    let index_path = std::env::var("PYFORGE_INDEX_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| JsonStore::default_path());
    let mut store = JsonStore::open(index_path).map_err(|e| {
        eprintln!("{}", e);
        2
    })?;

    match store.remove_project(&args.name) {
        Ok(()) => {
            let msg = t!("untrack.success", &args.name);
            if cli.json {
                println!("{}", json::success(serde_json::json!({ "removed": args.name })));
            }
            eprintln!("{}", msg);
            Ok(())
        }
        Err(IndexError::ProjectNotFound(name)) => {
            let msg = t!("untrack.not_found", &name);
            if cli.json {
                println!("{}", json::error("INDEX", "INDEX_PROJECT_NOT_FOUND", &msg));
            }
            eprintln!("{}", msg);
            Err(1)
        }
        Err(e) => {
            eprintln!("{}", e);
            Err(2)
        }
    }
}
