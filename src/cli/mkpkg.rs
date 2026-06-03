//! `pyforge mkpkg` 命令实现。

use super::Cli;
use crate::output::json;
use crate::t;
use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
pub struct MkpkgArgs {
    /// 子包名称（支持 a.b.c 嵌套写法）
    #[arg(required = true)]
    pub packages: Vec<String>,

    /// 基础路径（默认当前目录）
    #[arg(long)]
    pub base: Option<String>,

    /// 仅显示将创建的路径，不实际写入
    #[arg(long)]
    pub dry_run: bool,
}

/// 校验包名不包含路径遍历组件。
fn validate_package_name(pkg: &str) -> Result<(), String> {
    for component in pkg.split('.') {
        if component.is_empty() || component == ".." || component.contains('/') || component.contains('\\') {
            return Err(format!("invalid package name: {}", pkg));
        }
    }
    Ok(())
}

pub fn handle(cli: &Cli, args: &MkpkgArgs) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::mkpkg").entered();

    if args.packages.is_empty() {
        let msg = t!("mkpkg.no_name");
        if cli.json {
            println!("{}", json::error("CLI", "CLI_INVALID_ARG", &msg));
        }
        eprintln!("{}", msg);
        return Err(1);
    }

    let base = match &args.base {
        Some(b) => PathBuf::from(b),
        None => std::env::current_dir().map_err(|e| {
            eprintln!("{}", e);
            2
        })?,
    };

    let mut created = Vec::new();
    let mut skipped = Vec::new();
    let mut would_create = Vec::new();

    for pkg in &args.packages {
        if let Err(e) = validate_package_name(pkg) {
            if cli.json {
                println!("{}", json::error("CLI", "CLI_INVALID_ARG", &e));
            }
            eprintln!("{}", e);
            return Err(1);
        }
        let rel_path = pkg.replace('.', std::path::MAIN_SEPARATOR_STR);
        let dir = base.join(&rel_path);
        let init_file = dir.join("__init__.py");

        if args.dry_run {
            would_create.push(init_file.display().to_string());
            continue;
        }

        // 创建目录链（含中间目录的 __init__.py）
        create_intermediate_inits(&base, &rel_path)?;

        if init_file.exists() {
            skipped.push(init_file.display().to_string());
            eprintln!("{}", t!("mkpkg.skipped", &init_file.display().to_string()));
        } else {
            if let Some(parent) = init_file.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    eprintln!("{}", e);
                    2
                })?;
            }
            fs::write(&init_file, "").map_err(|e| {
                eprintln!("{}", e);
                2
            })?;
            created.push(init_file.display().to_string());
            eprintln!("{}", t!("mkpkg.created", &init_file.display().to_string()));
        }
    }

    if cli.json {
        println!(
            "{}",
            json::success(serde_json::json!({
                "created": created,
                "skipped": skipped,
                "would_create": would_create,
            }))
        );
    } else if args.dry_run {
        for p in &would_create {
            eprintln!("{}", t!("mkpkg.dry_run", p));
        }
    }

    Ok(())
}

/// 对于 `a.b.c` 这样的嵌套包，为中间层级也创建 `__init__.py`。
fn create_intermediate_inits(base: &Path, rel_path: &str) -> Result<(), i32> {
    let components: Vec<&str> = rel_path
        .split(std::path::MAIN_SEPARATOR)
        .collect();

    // 中间层级（不含最后一层，最后一层由调用方处理）
    for i in 0..components.len().saturating_sub(1) {
        let intermediate = base.join(components[..=i].join(std::path::MAIN_SEPARATOR_STR));
        let init = intermediate.join("__init__.py");
        if !init.exists() {
            fs::create_dir_all(&intermediate).map_err(|e| {
                eprintln!("{}", e);
                2
            })?;
            fs::write(&init, "").map_err(|e| {
                eprintln!("{}", e);
                2
            })?;
        }
    }
    Ok(())
}
