//! `pyforge outdated` 命令实现。

use super::Cli;
use crate::index::store::{IndexStore, JsonStore};
use crate::output::json;
use crate::t;
use crate::uv::bridge;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct OutdatedArgs {
    /// 项目名称（可选，不传则遍历所有项目）
    pub name: Option<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
struct OutdatedProject {
    name: String,
    path: String,
    outdated: Vec<OutdatedDepEntry>,
}

#[derive(serde::Serialize, Clone, Debug)]
struct OutdatedDepEntry {
    name: String,
    current: String,
    latest: String,
}

pub fn handle(cli: &Cli, args: &OutdatedArgs) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::outdated").entered();

    let index_path = std::env::var("PYFORGE_INDEX_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| JsonStore::default_path());
    let store = JsonStore::open(index_path).map_err(|e| {
        eprintln!("{}", e);
        2
    })?;

    // 确定要检查的项目列表
    let projects = if let Some(ref name) = args.name {
        match store.get_project(name) {
            Some(p) => vec![p],
            None => {
                eprintln!("{}", t!("info.not_found", name));
                return Err(1);
            }
        }
    } else {
        store.list_projects()
    };

    if projects.is_empty() {
        if cli.json {
            println!("{}", json::success(serde_json::json!({ "projects": [] })));
        }
        eprintln!("{}", t!("status.empty"));
        return Ok(());
    }

    let mut results: Vec<OutdatedProject> = Vec::new();

    for project in &projects {
        let path = PathBuf::from(&project.path);
        if !path.exists() {
            continue;
        }

        match bridge::outdated(&path) {
            Ok(deps) => {
                let entries: Vec<OutdatedDepEntry> = deps
                    .into_iter()
                    .map(|d| OutdatedDepEntry {
                        name: d.name,
                        current: d.current,
                        latest: d.latest,
                    })
                    .collect();
                results.push(OutdatedProject {
                    name: project.name.clone(),
                    path: project.path.clone(),
                    outdated: entries,
                });
            }
            Err(crate::uv::error::UvError::NotInstalled) => {
                eprintln!("{}", t!("outdated.uv_missing"));
                if cli.json {
                    println!(
                        "{}",
                        json::error("UV", "UV_NOT_INSTALLED", &t!("outdated.uv_missing"))
                    );
                }
                return Err(1);
            }
            Err(crate::uv::error::UvError::ParseError(ref msg)) => {
                eprintln!("{}", t!("outdated.parse_error"));
                if cli.json {
                    println!(
                        "{}",
                        json::error("UV", "UV_OUTDATED_ERROR", &t!("outdated.parse_error"))
                    );
                }
                tracing::warn!("uv outdated parse error for {}: {}", project.name, msg);
                return Err(2);
            }
            Err(e) => {
                eprintln!("{}", e);
                return Err(2);
            }
        }
    }

    // 输出结果
    if cli.json {
        println!(
            "{}",
            json::success(serde_json::json!({ "projects": &results }))
        );
    } else {
        let has_any = results.iter().any(|p| !p.outdated.is_empty());
        if !has_any {
            eprintln!("{}", t!("outdated.all_up_to_date"));
        } else {
            for project in &results {
                if project.outdated.is_empty() {
                    continue;
                }
                println!("{}:", project.name);
                for dep in &project.outdated {
                    println!("{}", t!("outdated.item", dep.name, dep.current, dep.latest));
                }
            }
        }
    }

    Ok(())
}
