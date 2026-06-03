//! `pyforge list` 命令实现。

use super::Cli;
use crate::output::json;
use crate::t;

use crate::index::store::{IndexStore, JsonStore};
use std::path::PathBuf;

pub fn handle(cli: &Cli) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::list").entered();

    let index_path = std::env::var("PYFORGE_INDEX_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| JsonStore::default_path());
    let store = JsonStore::open(index_path).map_err(|e| {
        eprintln!("{}", e);
        2
    })?;

    let mut projects = store.list_projects();
    projects.sort_by(|a, b| a.name.cmp(&b.name));

    if projects.is_empty() {
        if cli.json {
            println!("{}", json::success(serde_json::json!({ "projects": [] })));
        } else {
            println!("{}", t!("list.empty"));
        }
        return Ok(());
    }

    if cli.json {
        println!(
            "{}",
            json::success(serde_json::json!({ "projects": projects }))
        );
    } else {
        for p in &projects {
            println!("{}", t!("list.item", &p.name, &p.path));
        }
    }

    Ok(())
}
