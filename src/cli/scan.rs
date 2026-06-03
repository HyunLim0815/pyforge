//! `pyforge scan` 命令实现。

use super::Cli;
use crate::index::scanner;
use crate::index::store::{IndexStore, JsonStore};
use crate::index::types::ProjectInfo;
use crate::output::json;
use crate::t;
use clap::Args;
use serde_json::json as json_value;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ScanArgs {
    /// 扫描目录
    pub directory: PathBuf,

    /// 最大递归深度
    #[arg(long)]
    pub depth: Option<usize>,

    /// 自动注册发现的项目
    #[arg(long)]
    pub auto_track: bool,
}

pub fn handle(cli: &Cli, args: &ScanArgs) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::scan").entered();

    if !args.directory.exists() || !args.directory.is_dir() {
        let msg = t!("track.path_not_found", args.directory.display().to_string());
        if cli.json {
            println!("{}", json::error("SCAN", "SCAN_DIR_NOT_FOUND", &msg));
        }
        eprintln!("{}", msg);
        return Err(1);
    }

    eprintln!(
        "{}",
        t!("scan.progress", args.directory.display().to_string())
    );
    let projects = scanner::scan(&args.directory, args.depth);

    if projects.is_empty() {
        if cli.json {
            println!("{}", json::success(json_value!({ "projects": [] })));
        } else {
            println!("{}", t!("scan.no_match"));
        }
        return Ok(());
    }

    let mut tracked = 0usize;
    let mut skipped = 0usize;

    if args.auto_track {
        let index_path = std::env::var("PYFORGE_INDEX_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| JsonStore::default_path());
        let mut store = JsonStore::open(index_path).map_err(|e| {
            eprintln!("{}", e);
            2
        })?;

        for p in &projects {
            let info = ProjectInfo {
                name: p.name.clone(),
                path: p.path.display().to_string(),
                python_version: p.python_version.clone(),
                toolchain: p.toolchain.clone(),
                git_remote: None,
                git_branch: None,
                git_status: None,
                deps_count: None,
                created_at: None,
                last_modified: None,
                tags: vec![],
                description: None,
            };
            match store.add_project(info) {
                Ok(()) => tracked += 1,
                Err(crate::index::error::IndexError::AlreadyTracked(_)) => skipped += 1,
                Err(e) => {
                    eprintln!("{}", e);
                    return Err(2);
                }
            }
        }
    }

    if cli.json {
        let items: Vec<_> = projects
            .iter()
            .map(|p| {
                json_value!({
                    "name": p.name,
                    "path": p.path,
                    "confidence": format!("{:?}", p.confidence),
                    "python_version": p.python_version,
                    "toolchain": p.toolchain,
                })
            })
            .collect();
        println!(
            "{}",
            json::success(json_value!({
                "projects": items,
                "found": projects.len(),
                "tracked": tracked,
                "skipped": skipped,
            }))
        );
    } else {
        println!("{}", t!("scan.found", projects.len()));
        for p in &projects {
            println!("{}\t{}\t{:?}", p.name, p.path.display(), p.confidence);
        }
    }

    Ok(())
}
