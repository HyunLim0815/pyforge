//! `pyforge track` 命令实现。
//!
//! 注册项目到本地索引，采集元数据（Python 版本、工具链、Git 状态）。

use super::Cli;
use crate::output::json;
use crate::t;

use crate::index::detector;
use crate::index::store::{IndexStore, JsonStore};
use crate::index::types::ProjectInfo;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct TrackArgs {
    /// 项目路径
    pub path: PathBuf,

    /// 自定义项目名称（默认使用目录名）
    #[arg(long)]
    pub name: Option<String>,
}

pub fn handle(cli: &Cli, args: &TrackArgs) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::track").entered();

    // 路径验证
    if !args.path.exists() {
        let msg = t!("track.path_not_found", args.path.display().to_string());
        if cli.json {
            println!("{}", json::error("CLI", "CLI_INVALID_ARG", &msg));
        }
        eprintln!("{}", msg);
        return Err(1);
    }

    let canonical = args
        .path
        .canonicalize()
        .unwrap_or_else(|_| args.path.clone());
    let name = args.name.clone().unwrap_or_else(|| {
        canonical
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    });

    // 打开索引（支持 PYFORGE_INDEX_PATH 环境变量注入）
    let index_path = std::env::var("PYFORGE_INDEX_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| JsonStore::default_path());
    let mut store = JsonStore::open(index_path).map_err(|e| {
        eprintln!("{}", e);
        2
    })?;

    // 采集元数据
    let detection = detector::detect(&canonical);
    let created_at = chrono_now();
    let last_modified = modified_time(&canonical);
    let (git_remote, git_branch, git_status) = git_info(&canonical);

    let info = ProjectInfo {
        name: name.clone(),
        path: canonical.display().to_string(),
        python_version: detection.python_version,
        toolchain: detection.toolchain,
        git_remote,
        git_branch,
        git_status,
        deps_count: None,
        created_at: Some(created_at),
        last_modified,
        tags: vec![],
        description: None,
    };

    // 写入索引
    match store.add_project(info.clone()) {
        Ok(()) => {
            let msg = t!("track.success", &name);
            if cli.json {
                println!("{}", json::success(serde_json::json!({ "project": &info })));
            }
            eprintln!("{}", msg);
            Ok(())
        }
        Err(crate::index::error::IndexError::AlreadyTracked(n)) => {
            let msg = t!("track.already", &n);
            if cli.json {
                println!("{}", json::error("INDEX", "INDEX_ALREADY_TRACKED", &msg));
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

/// 将 Unix 时间戳转换为 ISO8601 UTC 字符串，正确处理闰年和每月天数。
fn epoch_secs_to_iso8601(secs: u64) -> String {
    let days = secs / 86400;
    let tod = secs % 86400;
    let h = tod / 3600;
    let m = (tod % 3600) / 60;
    let s = tod % 60;

    // 从儒略日推算年月日（正确处理闰年）
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
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn chrono_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    epoch_secs_to_iso8601(secs)
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

fn git_info(path: &PathBuf) -> (Option<String>, Option<String>, Option<String>) {
    match git2::Repository::open(path) {
        Ok(repo) => {
            let remote = repo
                .find_remote("origin")
                .ok()
                .and_then(|r| r.url().map(|u| u.to_string()));
            let branch = repo
                .head()
                .ok()
                .and_then(|h| h.shorthand().map(|s| s.to_string()));
            let status = {
                let mut opts = git2::StatusOptions::new();
                opts.include_untracked(false);
                let count = repo.statuses(Some(&mut opts)).map(|s| s.len()).unwrap_or(0);
                if count == 0 {
                    Some("clean".into())
                } else {
                    Some("dirty".into())
                }
            };
            (remote, branch, status)
        }
        Err(_) => (None, None, None),
    }
}
