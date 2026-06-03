//! uv CLI 子进程桥接。
//!
//! 生产路径调用 `uv init <name>`；测试路径可通过 `PYFORGE_UV_MOCK=success|missing|fail` 显式 mock。

use crate::uv::error::UvError;
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

/// 过期依赖信息。
#[derive(Debug, Clone, Deserialize)]
pub struct OutdatedDep {
    pub name: String,
    pub current: String,
    pub latest: String,
}

/// uv 版本信息。
#[derive(Debug, Clone, serde::Serialize)]
pub struct UvVersion {
    pub installed: bool,
    pub version: Option<String>,
}

/// 查询 uv 版本。未安装时返回 `installed: false`。
pub fn version() -> UvVersion {
    match std::env::var("PYFORGE_UV_MOCK").as_deref() {
        Ok("success") => {
            return UvVersion {
                installed: true,
                version: Some("0.6.0 (mock)".into()),
            }
        }
        Ok("missing") => {
            return UvVersion {
                installed: false,
                version: None,
            }
        }
        _ => {}
    }

    let uv_bin = std::env::var("PYFORGE_UV_BIN").unwrap_or_else(|_| "uv".into());
    match Command::new(&uv_bin).arg("--version").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            // uv 输出格式: "uv 0.6.0 (abc1234 2025-01-01)"
            let ver = stdout
                .strip_prefix("uv ")
                .unwrap_or(&stdout)
                .split_whitespace()
                .next()
                .unwrap_or(&stdout)
                .to_string();
            UvVersion {
                installed: true,
                version: Some(ver),
            }
        }
        _ => UvVersion {
            installed: false,
            version: None,
        },
    }
}

pub fn init(name: &str, cwd: &Path) -> Result<(), UvError> {
    match std::env::var("PYFORGE_UV_MOCK").as_deref() {
        Ok("success") => return mock_success(name, cwd),
        Ok("missing") => return Err(UvError::NotInstalled),
        Ok("fail") => return Err(UvError::InitFailed("mock failure".into())),
        _ => {}
    }

    let uv_bin = std::env::var("PYFORGE_UV_BIN").unwrap_or_else(|_| "uv".into());
    let output = Command::new(uv_bin)
        .arg("init")
        .arg(name)
        .current_dir(cwd)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(UvError::InitFailed(stderr))
    }
}

fn mock_success(name: &str, cwd: &Path) -> Result<(), UvError> {
    let project_dir = cwd.join(name);
    std::fs::create_dir_all(project_dir.join("src"))?;
    std::fs::write(
        project_dir.join("pyproject.toml"),
        format!("[project]\nname = \"{}\"\nversion = \"0.1.0\"\n", name),
    )?;
    Ok(())
}

/// 调用 `uv outdated` 并解析 stdout，返回过期依赖列表。
///
/// Mock 支持（`PYFORGE_UV_MOCK`）：
/// - `"outdated:<csv>"` — 返回解析后的 `OutdatedDep` 列表，csv 格式 `name:current:latest,...`
/// - `"outdated_clean"` — 返回空列表（所有依赖为最新）
/// - `"outdated_parse_error"` — 返回 `UvError::ParseError`
/// - `"missing"` — 返回 `UvError::NotInstalled`
pub fn outdated(project_path: &Path) -> Result<Vec<OutdatedDep>, UvError> {
    // Test mock path
    match std::env::var("PYFORGE_UV_MOCK").as_deref() {
        Ok("outdated_clean") => return Ok(vec![]),
        Ok("outdated_parse_error") => {
            return Err(UvError::ParseError("mock parse failure".into()))
        }
        Ok("missing") => return Err(UvError::NotInstalled),
        Ok(mock) if mock.starts_with("outdated:") => {
            let csv = &mock["outdated:".len()..];
            if csv.is_empty() {
                return Ok(vec![]);
            }
            let mut deps = Vec::new();
            for entry in csv.split(',') {
                let parts: Vec<&str> = entry.split(':').collect();
                if parts.len() != 3 {
                    return Err(UvError::ParseError(format!("invalid mock data: {}", entry)));
                }
                deps.push(OutdatedDep {
                    name: parts[0].to_string(),
                    current: parts[1].to_string(),
                    latest: parts[2].to_string(),
                });
            }
            return Ok(deps);
        }
        _ => {}
    }

    let uv_bin = std::env::var("PYFORGE_UV_BIN").unwrap_or_else(|_| "uv".into());
    let output = Command::new(uv_bin)
        .arg("outdated")
        .current_dir(project_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(UvError::ParseError(stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_outdated_output(&stdout)
}

/// 解析 `uv outdated` 的表格输出。
///
/// 格式示例：
/// ```text
/// Package       Current  Latest
/// fastapi       0.100.0  0.109.0
/// uvicorn       0.23.0   0.27.0
/// ```
fn parse_outdated_output(output: &str) -> Result<Vec<OutdatedDep>, UvError> {
    let lines: Vec<&str> = output.lines().collect();

    // 空输出 = 所有依赖为最新
    if lines.is_empty() || (lines.len() == 1 && lines[0].trim().is_empty()) {
        return Ok(vec![]);
    }

    // 跳过表头行和分隔符行
    let data_lines: Vec<&str> = lines
        .iter()
        .skip_while(|l| {
            let trimmed = l.trim();
            trimmed.is_empty()
                || trimmed.starts_with("Package")
                || trimmed.starts_with('-')
                || trimmed.starts_with('=')
        })
        .copied()
        .collect();

    // 只有表头没有数据行 → 全部最新
    if data_lines.is_empty() {
        return Ok(vec![]);
    }

    let mut deps = Vec::new();
    for line in &data_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(UvError::ParseError(format!(
                "无法解析行: {}",
                trimmed
            )));
        }
        deps.push(OutdatedDep {
            name: parts[0].to_string(),
            current: parts[1].to_string(),
            latest: parts[2].to_string(),
        });
    }

    Ok(deps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_output_returns_empty() {
        assert!(parse_outdated_output("").unwrap().is_empty());
        assert!(parse_outdated_output("\n").unwrap().is_empty());
    }

    #[test]
    fn parse_header_only_returns_empty() {
        let output = "Package       Current  Latest\n";
        assert!(parse_outdated_output(output).unwrap().is_empty());
    }

    #[test]
    fn parse_table_with_deps() {
        let output = "Package       Current  Latest\nfastapi       0.100.0  0.109.0\nuvicorn       0.23.0   0.27.0\n";
        let deps = parse_outdated_output(output).unwrap();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].name, "fastapi");
        assert_eq!(deps[0].current, "0.100.0");
        assert_eq!(deps[0].latest, "0.109.0");
        assert_eq!(deps[1].name, "uvicorn");
        assert_eq!(deps[1].current, "0.23.0");
        assert_eq!(deps[1].latest, "0.27.0");
    }

    #[test]
    fn parse_malformed_line_returns_error() {
        let output = "Package       Current  Latest\nbad_line\n";
        let result = parse_outdated_output(output);
        assert!(result.is_err());
        match result.unwrap_err() {
            UvError::ParseError(_) => {}
            other => panic!("expected ParseError, got: {:?}", other),
        }
    }

    // ── 扩展单元测试 (P2) ──

    #[test]
    fn parse_trailing_whitespace_handled() {
        let output = "Package       Current  Latest\nfastapi       0.100.0  0.109.0   \n";
        let deps = parse_outdated_output(output).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "fastapi");
    }

    #[test]
    fn parse_mixed_blank_lines_handled() {
        let output = "\nPackage       Current  Latest\n\nfastapi       0.100.0  0.109.0\n\nuvicorn       0.23.0   0.27.0\n\n";
        let deps = parse_outdated_output(output).unwrap();
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn parse_data_without_header() {
        // uv 可能不输出表头（格式变更场景）
        let output = "fastapi       0.100.0  0.109.0\n";
        let deps = parse_outdated_output(output).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "fastapi");
    }

    #[test]
    fn mock_outdated_empty_csv_returns_empty() {
        // "outdated:" 前缀但空 CSV
        std::env::set_var("PYFORGE_UV_MOCK", "outdated:");
        let result = outdated(std::path::Path::new("."));
        std::env::remove_var("PYFORGE_UV_MOCK");
        assert!(result.unwrap().is_empty());
    }
}
