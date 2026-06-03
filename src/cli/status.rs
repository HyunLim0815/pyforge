//! `pyforge status` 命令实现。

use super::Cli;
use crate::index::store::{IndexStore, JsonStore};
use crate::output::json;
use crate::t;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct StatusArgs {}

#[derive(serde::Serialize, Clone, Debug)]
struct ProjectStatus {
    name: String,
    path: String,
    git_status: String,
    last_modified: Option<String>,
    deps_outdated: String,
    status: String,
}

pub async fn handle(cli: &Cli, _args: &StatusArgs) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::status").entered();

    let index_path = std::env::var("PYFORGE_INDEX_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| JsonStore::default_path());
    let store = JsonStore::open(index_path).map_err(|e| {
        eprintln!("{}", e);
        2
    })?;

    let projects = store.list_projects();

    if projects.is_empty() {
        let msg = t!("status.empty");
        if cli.json {
            println!("{}", json::success(serde_json::json!({ "projects": [] })));
        }
        eprintln!("{}", msg);
        return Ok(());
    }

    // tokio 并发处理 git 状态查询
    let handles: Vec<_> = projects
        .into_iter()
        .map(|p| {
            tokio::task::spawn_blocking(move || {
                let path = PathBuf::from(&p.path);
                if !path.exists() {
                    return ProjectStatus {
                        name: p.name,
                        path: p.path,
                        git_status: "missing".into(),
                        last_modified: None,
                        deps_outdated: "unknown".into(),
                        status: "missing".into(),
                    };
                }
                let git_status = git_dirty_status(&path);
                let last_modified = modified_time(&path);
                let status = if git_status == "dirty" {
                    "dirty".into()
                } else {
                    "ok".into()
                };
                ProjectStatus {
                    name: p.name,
                    path: p.path,
                    git_status,
                    last_modified,
                    deps_outdated: "unknown".into(),
                    status,
                }
            })
        })
        .collect();

    let mut statuses = Vec::with_capacity(handles.len());
    for h in handles {
        if let Ok(s) = h.await {
            statuses.push(s);
        }
    }

    if cli.json {
        println!(
            "{}",
            json::success(serde_json::json!({ "projects": &statuses }))
        );
    } else {
        for s in &statuses {
            let status_icon = match s.status.as_str() {
                "dirty" => "[dirty]",
                "missing" => "[missing]",
                _ => "[ok]",
            };
            let modified = s.last_modified.as_deref().unwrap_or("-");
            println!(
                "{:<20} {:<10} {:<22} deps_outdated={}",
                s.name, status_icon, modified, s.deps_outdated
            );
        }
    }

    Ok(())
}

fn git_dirty_status(path: &PathBuf) -> String {
    match git2::Repository::open(path) {
        Ok(repo) => {
            let mut opts = git2::StatusOptions::new();
            opts.include_untracked(false);
            let count = repo.statuses(Some(&mut opts)).map(|s| s.len()).unwrap_or(0);
            if count == 0 {
                "clean".into()
            } else {
                "dirty".into()
            }
        }
        Err(_) => "no_git".into(),
    }
}

/// 将 Unix 时间戳转换为 ISO8601 UTC 字符串，正确处理闰年和每月天数。
fn epoch_secs_to_iso8601(secs: u64) -> String {
    let days = secs / 86400;
    let tod = secs % 86400;
    let h = tod / 3600;
    let m = (tod % 3600) / 60;
    let s = tod % 60;

    let mut y = 1970u64;
    let mut remaining = days;
    loop {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let month_days: [u64; 12] = [
        31,
        if is_leap_year(y) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut mo = 1u64;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        mo += 1;
    }
    let d = remaining + 1;
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn modified_time(path: &PathBuf) -> Option<String> {
    std::fs::metadata(path).ok()?.modified().ok().map(|t| {
        let secs = t
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        epoch_secs_to_iso8601(secs)
    })
}
