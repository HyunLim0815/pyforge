//! Python 项目检测器。
//!
//! 置信度分级：High(pyproject.toml) / Medium(setup.py) / Low(.py only)

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// 检测结果。
pub struct Detection {
    pub confidence: Confidence,
    pub python_version: Option<String>,
    pub toolchain: Option<String>,
}

/// 检测指定路径是否为 Python 项目。
pub fn detect(project_path: &Path) -> Detection {
    let has_pyproject = project_path.join("pyproject.toml").exists();
    let has_setup_py = project_path.join("setup.py").exists();

    let confidence = if has_pyproject {
        Confidence::High
    } else if has_setup_py {
        Confidence::Medium
    } else {
        Confidence::Low
    };

    let python_version = if has_pyproject {
        parse_python_requires(project_path)
    } else {
        None
    };

    let toolchain = detect_toolchain(project_path);

    Detection {
        confidence,
        python_version,
        toolchain,
    }
}

/// 从 pyproject.toml 解析 requires-python。
fn parse_python_requires(project_path: &Path) -> Option<String> {
    let pyproject = project_path.join("pyproject.toml");
    let content = std::fs::read_to_string(pyproject).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("requires-python") {
            // 找到第一个 = 后的全部内容
            if let Some(pos) = trimmed.find('=') {
                let val = trimmed[pos + 1..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

/// 检测工具链：uv / poetry / setuptools / unknown。
fn detect_toolchain(project_path: &Path) -> Option<String> {
    if project_path.join("uv.lock").exists() {
        return Some("uv".into());
    }
    if project_path.join("poetry.lock").exists() {
        return Some("poetry".into());
    }
    if project_path.join("pyproject.toml").exists() {
        return Some("unknown".into());
    }
    if project_path.join("setup.py").exists() || project_path.join("setup.cfg").exists() {
        return Some("setuptools".into());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn high_confidence_for_pyproject() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("pyproject.toml"),
            "[project]\nname = \"test\"\nrequires-python = \">=3.10\"\n",
        )
        .unwrap();
        let d = detect(dir.path());
        assert_eq!(d.confidence, Confidence::High);
        assert_eq!(d.python_version.unwrap(), ">=3.10");
    }

    #[test]
    fn medium_confidence_for_setup_py() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("setup.py"), "").unwrap();
        let d = detect(dir.path());
        assert_eq!(d.confidence, Confidence::Medium);
    }

    #[test]
    fn uv_toolchain_detected() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("pyproject.toml"), "").unwrap();
        fs::write(dir.path().join("uv.lock"), "").unwrap();
        let d = detect(dir.path());
        assert_eq!(d.toolchain.unwrap(), "uv");
    }
}
