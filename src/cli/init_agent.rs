//! `pyforge init-agent` 命令实现。

use super::Cli;
use crate::agent::skill;
use crate::output::json;
use crate::t;
use clap::Args;

#[derive(Args, Debug)]
pub struct InitAgentArgs {
    /// 跳过交互确认，直接复制
    #[arg(long, default_value_t = false)]
    pub yes: bool,
}

#[derive(serde::Serialize, Clone, Debug)]
struct InstallResult {
    target: String,
    path: String,
    status: String,
}

pub fn handle(cli: &Cli, args: &InitAgentArgs) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::init-agent").entered();

    // 先确保 ~/.pyforge/SKILL.md 存在
    match skill::ensure_skill_md() {
        Ok(true) => eprintln!("{}", t!("agent.skill_written", skill::default_skill_path().display())),
        Ok(false) => {}
        Err(e) => {
            eprintln!("{}", e);
            return Err(2);
        }
    }

    let targets = skill::known_targets();
    let mut results: Vec<InstallResult> = Vec::new();

    for target in &targets {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let dest = home.join(target.dir).join(target.filename);

        // 交互确认（Plain 模式且未 --yes）
        if !cli.json && !args.yes {
            eprintln!(
                "{} → {}? [Y/n]",
                target.name,
                dest.display()
            );
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() {
                let trimmed = input.trim();
                if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("y") {
                    results.push(InstallResult {
                        target: target.name.to_string(),
                        path: dest.display().to_string(),
                        status: "skipped".to_string(),
                    });
                    continue;
                }
            }
        }

        match skill::copy_to_target(target) {
            Ok(skill::CopyResult::Written(p)) => {
                eprintln!("{}: {}", t!("agent.skill_written", p.display()), target.name);
                results.push(InstallResult {
                    target: target.name.to_string(),
                    path: p.display().to_string(),
                    status: "written".to_string(),
                });
            }
            Ok(skill::CopyResult::Skipped) => {
                results.push(InstallResult {
                    target: target.name.to_string(),
                    path: dest.display().to_string(),
                    status: "skipped".to_string(),
                });
            }
            Err(e) => {
                eprintln!("{}: {}", target.name, e);
                results.push(InstallResult {
                    target: target.name.to_string(),
                    path: dest.display().to_string(),
                    status: "error".to_string(),
                });
            }
        }
    }

    if cli.json {
        println!(
            "{}",
            json::success(serde_json::json!({ "results": &results }))
        );
    }

    Ok(())
}
