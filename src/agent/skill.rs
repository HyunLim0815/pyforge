//! SKILL.md 模板加载与写入。

use std::path::Path;

/// 内嵌的 SKILL.md 模板内容。
pub const SKILL_MD: &str = include_str!("../../skills/pyforge.md");

/// 确保 `~/.pyforge/SKILL.md` 存在；缺失则写入。
/// 返回 true 表示新写入，false 表示已存在。
pub fn ensure_skill_md() -> Result<bool, std::io::Error> {
    let skill_path = default_skill_path();
    if skill_path.exists() {
        return Ok(false);
    }
    if let Some(parent) = skill_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&skill_path, SKILL_MD)?;
    Ok(true)
}

/// 默认 SKILL.md 路径：`~/.pyforge/SKILL.md`
pub fn default_skill_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".pyforge")
        .join("SKILL.md")
}

/// 已知 AI 工具目标目录。
pub struct AgentTarget {
    pub name: &'static str,
    pub dir: &'static str,
    pub filename: &'static str,
}

/// 返回已知 AI 工具目标列表。
pub fn known_targets() -> Vec<AgentTarget> {
    vec![
        AgentTarget {
            name: "Claude Code",
            dir: ".claude/skills",
            filename: "pyforge.md",
        },
        AgentTarget {
            name: "Codex CLI",
            dir: ".codex/skills",
            filename: "pyforge.md",
        },
        AgentTarget {
            name: "Windsurf",
            dir: ".windsurf/skills",
            filename: "pyforge.md",
        },
    ]
}

/// 将 SKILL.md 复制到目标路径。返回 true 表示写入，false 表示内容相同跳过。
pub fn copy_to_target(target: &AgentTarget) -> Result<CopyResult, std::io::Error> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let dest = home.join(target.dir).join(target.filename);

    if dest.exists() {
        let existing = std::fs::read_to_string(&dest)?;
        if existing == SKILL_MD {
            return Ok(CopyResult::Skipped);
        }
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&dest, SKILL_MD)?;
    Ok(CopyResult::Written(dest))
}

pub enum CopyResult {
    Written(std::path::PathBuf),
    Skipped,
}

/// 将 SKILL.md 写入指定路径（测试用）。
pub fn write_to(path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, SKILL_MD)
}
