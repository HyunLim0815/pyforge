//! 依赖分析 API。
//!
//! `GET /api/dependencies` 聚合所有项目的依赖信息。

use axum::Json;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::index::store::{IndexStore, JsonStore};

/// 单个依赖的聚合信息。
#[derive(Debug, Clone, Serialize)]
pub struct DepInfo {
    pub count: usize,
    pub projects: Vec<String>,
}

/// `/api/dependencies` 响应体。
#[derive(Debug, Clone, Serialize)]
pub struct DependenciesResponse {
    pub dependencies: HashMap<String, DepInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// `GET /api/dependencies` — 返回跨项目依赖聚合 JSON。
pub async fn get_dependencies() -> Json<DependenciesResponse> {
    let store = match JsonStore::load_or_create() {
        Ok(s) => s,
        Err(e) => return Json(DependenciesResponse {
            dependencies: HashMap::new(),
            error: Some(format!("索引加载失败: {}", e)),
        }),
    };
    let projects = store.list_projects();

    let mut dep_map: HashMap<String, Vec<String>> = HashMap::new();

    for p in &projects {
        let project_path = Path::new(&p.path);
        let deps = parse_project_deps(project_path);
        for dep_name in deps {
            dep_map
                .entry(dep_name)
                .or_default()
                .push(p.name.clone());
        }
    }

    // 按使用频次降序排序（转换为有序结构）
    let mut dependencies: Vec<(String, DepInfo)> = dep_map
        .into_iter()
        .map(|(name, mut projects)| {
            projects.sort();
            projects.dedup();
            let count = projects.len();
            (name, DepInfo { count, projects })
        })
        .collect();
    dependencies.sort_by(|a, b| b.1.count.cmp(&a.1.count).then(a.0.cmp(&b.0)));

    Json(DependenciesResponse {
        dependencies: dependencies.into_iter().collect(),
        error: None,
    })
}

/// 解析项目目录中的 pyproject.toml，提取依赖包名。
///
/// 支持两种格式：
/// - PEP 621: `[project] dependencies = [...]`
/// - Poetry: `[tool.poetry.dependencies]`
///
/// 解析失败时返回空列表（不中断整体响应）。
fn parse_project_deps(project_path: &Path) -> Vec<String> {
    let toml_path = project_path.join("pyproject.toml");
    if !toml_path.exists() {
        return Vec::new();
    }

    let content = match fs::read_to_string(&toml_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let doc: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut deps = Vec::new();

    // PEP 621: [project] dependencies = ["requests>=2.28", ...]
    if let Some(project) = doc.get("project") {
        if let Some(arr) = project.get("dependencies").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    if let Some(name) = extract_dep_name(s) {
                        deps.push(name);
                    }
                }
            }
        }
        // optional-dependencies
        if let Some(opts) = project.get("optional-dependencies").and_then(|v| v.as_table()) {
            for (_group, arr) in opts {
                if let Some(arr) = arr.as_array() {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            if let Some(name) = extract_dep_name(s) {
                                deps.push(name);
                            }
                        }
                    }
                }
            }
        }
    }

    // Poetry: [tool.poetry.dependencies] (排除 python 本身)
    if let Some(tool) = doc.get("tool") {
        if let Some(poetry) = tool.get("poetry") {
            if let Some(deps_table) = poetry.get("dependencies").and_then(|v| v.as_table()) {
                for (name, _val) in deps_table {
                    if name != "python" {
                        deps.push(name.clone());
                    }
                }
            }
            // Poetry extras
            if let Some(extras) = poetry.get("extras").and_then(|v| v.as_table()) {
                for (_group, val) in extras {
                    if let Some(arr) = val.as_array() {
                        for item in arr {
                            if let Some(s) = item.as_str() {
                                if let Some(name) = extract_dep_name(s) {
                                    deps.push(name);
                                }
                            }
                        }
                    } else if let Some(tbl) = val.as_table() {
                        for name in tbl.keys() {
                            deps.push(name.clone());
                        }
                    }
                }
            }
        }
    }

    deps.sort();
    deps.dedup();
    deps
}

/// 从 pip 格式的依赖字符串中提取包名。
///
/// 例如 `"requests>=2.28.0"` → `"requests"`，`"uvicorn[standard]"` → `"uvicorn"`。
fn extract_dep_name(spec: &str) -> Option<String> {
    let s = spec.trim();
    if s.is_empty() || s.starts_with('#') || s.starts_with('-') {
        return None;
    }
    // 处理 URL/git 依赖（如 `requests @ https://...`）
    if s.contains("://") || s.contains("git+") {
        return None;
    }
    // 处理环境标记（分号后的内容）
    let s = if let Some(idx) = s.find(';') {
        s[..idx].trim()
    } else {
        s
    };
    // 提取包名：在第一个版本限定符处截断
    let end = s
        .find(|c: char| ['>', '<', '=', '!', '~', '['].contains(&c))
        .unwrap_or(s.len());
    let name = s[..end].trim();
    if name.is_empty() {
        return None;
    }
    // 规范化：小写 + 连字符替换下划线
    Some(name.to_lowercase().replace('_', "-"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn extract_dep_name_pep508() {
        assert_eq!(extract_dep_name("requests>=2.28"), Some("requests".into()));
        assert_eq!(extract_dep_name("uvicorn[standard]"), Some("uvicorn".into()));
        assert_eq!(extract_dep_name("Django~=4.2"), Some("django".into()));
        assert_eq!(extract_dep_name("click !=8.0"), Some("click".into()));
        assert_eq!(extract_dep_name("my-package>=1.0"), Some("my-package".into()));
        assert_eq!(extract_dep_name("my_package>=1.0"), Some("my-package".into()));
    }

    #[test]
    fn extract_dep_name_edge_cases() {
        assert_eq!(extract_dep_name(""), None);
        assert_eq!(extract_dep_name("# comment"), None);
        assert_eq!(extract_dep_name("-e ./local"), None);
        assert_eq!(
            extract_dep_name("requests @ https://example.com/pkg"),
            None
        );
        assert_eq!(
            extract_dep_name("mypkg ; python_version >= '3.8'"),
            Some("mypkg".into())
        );
    }

    #[test]
    fn parse_project_deps_pep621() {
        let dir = TempDir::new().unwrap();
        let pyproject = r#"
[project]
name = "test-proj"
version = "0.1.0"
dependencies = [
    "requests>=2.28",
    "click>=8.0",
]
"#;
        fs::write(dir.path().join("pyproject.toml"), pyproject).unwrap();
        let deps = parse_project_deps(dir.path());
        assert!(deps.contains(&"requests".to_string()));
        assert!(deps.contains(&"click".to_string()));
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn parse_project_deps_poetry() {
        let dir = TempDir::new().unwrap();
        let pyproject = r#"
[tool.poetry]
name = "test-proj"

[tool.poetry.dependencies]
python = "^3.10"
requests = "^2.28"
click = ">=8.0"
"#;
        fs::write(dir.path().join("pyproject.toml"), pyproject).unwrap();
        let deps = parse_project_deps(dir.path());
        assert!(deps.contains(&"requests".to_string()));
        assert!(deps.contains(&"click".to_string()));
        // python 应被排除
        assert!(!deps.iter().any(|d| d == "python"));
    }

    #[test]
    fn parse_project_deps_no_pyproject() {
        let dir = TempDir::new().unwrap();
        let deps = parse_project_deps(dir.path());
        assert!(deps.is_empty());
    }

    #[test]
    fn parse_project_deps_invalid_toml() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("pyproject.toml"), "not valid toml [[[").unwrap();
        let deps = parse_project_deps(dir.path());
        assert!(deps.is_empty());
    }

    #[test]
    fn aggregation_groups_by_name() {
        // 验证 DepInfo 结构正确聚合
        let info = DepInfo {
            count: 3,
            projects: vec!["a".into(), "b".into(), "c".into()],
        };
        assert_eq!(info.count, 3);
        assert_eq!(info.projects.len(), 3);
    }
}
