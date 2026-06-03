//! 跨平台打开目录/终端/IDE 的命令分发。
//!
//! 所有命令通过白名单校验，仅支持预定义的 action。

use std::path::Path;
use std::process::Command;

/// 支持的操作类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAction {
    /// 在文件管理器中打开目录。
    Dir,
    /// 打开终端并 cd 到目录。
    Terminal,
    /// 在 VS Code 中打开。
    VsCode,
}

impl OpenAction {
    /// 从字符串解析 action，不在白名单内返回 None。
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "dir" => Some(Self::Dir),
            "terminal" => Some(Self::Terminal),
            "vscode" => Some(Self::VsCode),
            _ => None,
        }
    }
}

/// 执行打开操作。
///
/// # Safety
/// - `project_path` 必须是已验证存在的目录。
/// - `action` 已通过白名单校验。
pub fn execute(action: OpenAction, project_path: &Path) -> Result<(), String> {
    let path_str = project_path
        .to_str()
        .ok_or_else(|| "项目路径包含无效字符".to_string())?;

    match action {
        OpenAction::Dir => open_directory(path_str),
        OpenAction::Terminal => open_terminal(path_str),
        OpenAction::VsCode => open_vscode(path_str),
    }
}

#[cfg(target_os = "macos")]
fn open_directory(path: &str) -> Result<(), String> {
    Command::new("open")
        .arg(path)
        .status()
        .map_err(|e| format!("打开目录失败: {}", e))
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err("打开目录失败".into())
            }
        })
}

#[cfg(target_os = "linux")]
fn open_directory(path: &str) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(path)
        .status()
        .map_err(|e| format!("打开目录失败: {}", e))
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err("打开目录失败".into())
            }
        })
}

#[cfg(target_os = "windows")]
fn open_directory(path: &str) -> Result<(), String> {
    Command::new("explorer")
        .arg(path)
        .status()
        .map_err(|e| format!("打开目录失败: {}", e))
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err("打开目录失败".into())
            }
        })
}

#[cfg(target_os = "macos")]
fn open_terminal(path: &str) -> Result<(), String> {
    Command::new("open")
        .args(["-a", "Terminal", path])
        .status()
        .map_err(|e| format!("打开终端失败: {}", e))
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err("打开终端失败".into())
            }
        })
}

#[cfg(target_os = "linux")]
fn open_terminal(path: &str) -> Result<(), String> {
    // 尝试常见终端模拟器
    for terminal in &[
        "x-terminal-emulator",
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
    ] {
        if Command::new(terminal)
            .args(["--working-directory", path])
            .status()
            .is_ok()
        {
            return Ok(());
        }
    }
    Err("未找到可用的终端模拟器".into())
}

#[cfg(target_os = "windows")]
fn open_terminal(path: &str) -> Result<(), String> {
    Command::new("cmd")
        .args(["/C", "start", "cmd", "/K", &format!("cd /d {}", path)])
        .status()
        .map_err(|e| format!("打开终端失败: {}", e))
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err("打开终端失败".into())
            }
        })
}

fn open_vscode(path: &str) -> Result<(), String> {
    Command::new("code")
        .arg(path)
        .status()
        .map_err(|e| format!("打开 VS Code 失败: {}", e))
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err("打开 VS Code 失败，请确认已安装 code 命令".into())
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_action_from_str_valid() {
        assert_eq!(OpenAction::from_str("dir"), Some(OpenAction::Dir));
        assert_eq!(OpenAction::from_str("terminal"), Some(OpenAction::Terminal));
        assert_eq!(OpenAction::from_str("vscode"), Some(OpenAction::VsCode));
    }

    #[test]
    fn open_action_from_str_invalid() {
        assert_eq!(OpenAction::from_str("rm"), None);
        assert_eq!(OpenAction::from_str("exec"), None);
        assert_eq!(OpenAction::from_str(""), None);
        assert_eq!(OpenAction::from_str("Dir"), None); // 大小写敏感
        assert_eq!(OpenAction::from_str("../../../etc/passwd"), None);
    }

    #[test]
    fn open_action_whitelist_completeness() {
        // 确保所有合法 action 都在白名单中
        let valid = ["dir", "terminal", "vscode"];
        for a in &valid {
            assert!(OpenAction::from_str(a).is_some(), "{} should be valid", a);
        }
    }
}
