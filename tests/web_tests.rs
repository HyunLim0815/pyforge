//! Web API 集成测试。
//!
//! 测试依赖解析逻辑和主题切换 HTML 内容。

use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

/// 辅助函数：创建临时项目目录并写入 pyproject.toml。
fn setup_project(dir: &std::path::Path, name: &str, pyproject_content: &str) -> String {
    let project_dir = dir.join(name);
    fs::create_dir_all(&project_dir).unwrap();
    fs::write(project_dir.join("pyproject.toml"), pyproject_content).unwrap();
    project_dir.to_string_lossy().to_string()
}

/// 从路径解析依赖（与 dependencies.rs 同逻辑的独立副本，用于集成测试）。
fn parse_deps_from_path(project_path: &std::path::Path) -> Vec<String> {
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
        if let Some(opts) = project
            .get("optional-dependencies")
            .and_then(|v| v.as_table())
        {
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

    if let Some(tool) = doc.get("tool") {
        if let Some(poetry) = tool.get("poetry") {
            if let Some(deps_table) = poetry.get("dependencies").and_then(|v| v.as_table()) {
                for (name, _val) in deps_table {
                    if name != "python" {
                        deps.push(name.clone());
                    }
                }
            }
        }
    }

    deps.sort();
    deps.dedup();
    deps
}

fn extract_dep_name(spec: &str) -> Option<String> {
    let s = spec.trim();
    if s.is_empty() || s.starts_with('#') || s.starts_with('-') {
        return None;
    }
    if s.contains("://") || s.contains("git+") {
        return None;
    }
    let s = if let Some(idx) = s.find(';') {
        s[..idx].trim()
    } else {
        s
    };
    let end = s
        .find(|c: char| ['>', '<', '=', '!', '~', '['].contains(&c))
        .unwrap_or(s.len());
    let name = s[..end].trim();
    if name.is_empty() {
        return None;
    }
    Some(name.to_lowercase().replace('_', "-"))
}

// ── 依赖解析测试 ──

#[test]
fn dependency_aggregates_shared_deps() {
    let tmp = TempDir::new().unwrap();

    let proj_a = setup_project(
        tmp.path(),
        "proj-a",
        r#"
[project]
name = "proj-a"
dependencies = ["requests>=2.28", "click>=8.0", "httpx>=0.24"]
"#,
    );

    let proj_b = setup_project(
        tmp.path(),
        "proj-b",
        r#"
[project]
name = "proj-b"
dependencies = ["requests>=2.28", "pydantic>=2.0"]
"#,
    );

    let deps_a = parse_deps_from_path(std::path::Path::new(&proj_a));
    let deps_b = parse_deps_from_path(std::path::Path::new(&proj_b));

    // proj-a: click, httpx, requests (sorted)
    assert_eq!(deps_a.len(), 3);
    assert!(deps_a.contains(&"requests".to_string()));
    assert!(deps_a.contains(&"click".to_string()));
    assert!(deps_a.contains(&"httpx".to_string()));

    // proj-b: pydantic, requests (sorted)
    assert_eq!(deps_b.len(), 2);
    assert!(deps_b.contains(&"requests".to_string()));
    assert!(deps_b.contains(&"pydantic".to_string()));

    // 聚合验证：requests 在 2 个项目中
    let mut dep_map: HashMap<String, Vec<String>> = HashMap::new();
    for (name, deps) in [("proj-a", &deps_a), ("proj-b", &deps_b)] {
        for dep in deps {
            dep_map
                .entry(dep.clone())
                .or_default()
                .push(name.to_string());
        }
    }

    assert_eq!(dep_map["requests"].len(), 2);
    assert_eq!(dep_map["click"].len(), 1);
    assert_eq!(dep_map["httpx"].len(), 1);
    assert_eq!(dep_map["pydantic"].len(), 1);
}

#[test]
fn dependency_handles_poetry_format() {
    let tmp = TempDir::new().unwrap();
    let proj = setup_project(
        tmp.path(),
        "poetry-proj",
        r#"
[tool.poetry]
name = "poetry-proj"

[tool.poetry.dependencies]
python = "^3.10"
fastapi = "^0.100"
uvicorn = {version = "^0.23", extras = ["standard"]}
"#,
    );

    let deps = parse_deps_from_path(std::path::Path::new(&proj));
    assert!(deps.contains(&"fastapi".to_string()));
    assert!(deps.contains(&"uvicorn".to_string()));
    assert!(!deps.iter().any(|d| d == "python"));
}

#[test]
fn dependency_handles_missing_pyproject() {
    let tmp = TempDir::new().unwrap();
    let empty_dir = tmp.path().join("empty-proj");
    fs::create_dir_all(&empty_dir).unwrap();

    let deps = parse_deps_from_path(&empty_dir);
    assert!(deps.is_empty());
}

#[test]
fn dependency_handles_invalid_toml() {
    let tmp = TempDir::new().unwrap();
    let proj = setup_project(tmp.path(), "bad-proj", "not valid toml [[[");

    let deps = parse_deps_from_path(std::path::Path::new(&proj));
    assert!(deps.is_empty());
}

#[test]
fn dependency_deduplicates_within_project() {
    let tmp = TempDir::new().unwrap();
    let proj = setup_project(
        tmp.path(),
        "dup-proj",
        r#"
[project]
name = "dup-proj"
dependencies = ["requests>=2.28"]

[project.optional-dependencies]
dev = ["requests>=2.28", "pytest>=7.0"]
"#,
    );

    let deps = parse_deps_from_path(std::path::Path::new(&proj));
    let requests_count = deps.iter().filter(|d| d.as_str() == "requests").count();
    assert_eq!(requests_count, 1);
    assert!(deps.contains(&"pytest".to_string()));
}

#[test]
fn dependency_optional_dependencies_parsed() {
    let tmp = TempDir::new().unwrap();
    let proj = setup_project(
        tmp.path(),
        "extras-proj",
        r#"
[project]
name = "extras-proj"
dependencies = ["requests>=2.28"]

[project.optional-dependencies]
dev = ["pytest>=7.0", "ruff>=0.1"]
docs = ["sphinx>=7.0"]
"#,
    );

    let deps = parse_deps_from_path(std::path::Path::new(&proj));
    assert!(deps.contains(&"requests".to_string()));
    assert!(deps.contains(&"pytest".to_string()));
    assert!(deps.contains(&"ruff".to_string()));
    assert!(deps.contains(&"sphinx".to_string()));
}

// ── 主题切换 HTML 测试 ──

#[test]
fn theme_toggle_in_all_pages() {
    let pages = [
        ("dashboard.html", include_str!("../src/web/dashboard.html")),
        ("projects.html", include_str!("../src/web/projects.html")),
        (
            "project_detail.html",
            include_str!("../src/web/project_detail.html"),
        ),
        (
            "dependencies.html",
            include_str!("../src/web/dependencies.html"),
        ),
    ];
    for (name, html) in &pages {
        assert!(
            html.contains("pyforge_theme"),
            "{} 应包含 pyforge_theme localStorage key",
            name
        );
        assert!(
            html.contains("data-theme"),
            "{} 应包含 data-theme 属性操作",
            name
        );
        assert!(html.contains("themeToggle"), "{} 应包含主题切换按钮", name);
    }
}

#[test]
fn theme_css_variables_in_all_pages() {
    let pages = [
        include_str!("../src/web/dashboard.html"),
        include_str!("../src/web/projects.html"),
        include_str!("../src/web/project_detail.html"),
        include_str!("../src/web/dependencies.html"),
    ];
    for html in &pages {
        assert!(
            html.contains(":root") && html.contains("--bg:"),
            "应定义暗色主题 CSS 变量"
        );
        assert!(
            html.contains("[data-theme=\"light\"]"),
            "应定义亮色主题 CSS 变量"
        );
    }
}

#[test]
fn dependencies_page_has_nav_links() {
    let html = include_str!("../src/web/dependencies.html");
    assert!(html.contains("href=\"/\""), "应有 Dashboard 链接");
    assert!(html.contains("href=\"/projects\""), "应有 Projects 链接");
}

#[test]
fn all_pages_have_nav_to_dependencies() {
    let pages = [
        ("dashboard.html", include_str!("../src/web/dashboard.html")),
        ("projects.html", include_str!("../src/web/projects.html")),
        (
            "project_detail.html",
            include_str!("../src/web/project_detail.html"),
        ),
    ];
    for (name, html) in &pages {
        assert!(
            html.contains("href=\"/dependencies\""),
            "{} 应有 Dependencies 导航链接",
            name
        );
    }
}

// ── i18n 测试 ──

#[test]
fn i18n_js_loaded_in_all_pages() {
    let pages = [
        ("dashboard.html", include_str!("../src/web/dashboard.html")),
        ("projects.html", include_str!("../src/web/projects.html")),
        (
            "project_detail.html",
            include_str!("../src/web/project_detail.html"),
        ),
        (
            "dependencies.html",
            include_str!("../src/web/dependencies.html"),
        ),
    ];
    for (name, html) in &pages {
        assert!(html.contains("src=\"/i18n.js\""), "{} 应引用 i18n.js", name);
        assert!(
            html.contains("PyForgeI18n.initI18n()"),
            "{} 应调用 initI18n()",
            name
        );
    }
}

#[test]
fn lang_toggle_in_all_pages() {
    let pages = [
        ("dashboard.html", include_str!("../src/web/dashboard.html")),
        ("projects.html", include_str!("../src/web/projects.html")),
        (
            "project_detail.html",
            include_str!("../src/web/project_detail.html"),
        ),
        (
            "dependencies.html",
            include_str!("../src/web/dependencies.html"),
        ),
    ];
    for (name, html) in &pages {
        assert!(
            html.contains("id=\"langToggle\""),
            "{} 应包含语言切换按钮",
            name
        );
    }
}

#[test]
fn data_i18n_attributes_in_all_pages() {
    let pages = [
        (
            "dashboard.html",
            include_str!("../src/web/dashboard.html"),
            "dashboard.title",
        ),
        (
            "projects.html",
            include_str!("../src/web/projects.html"),
            "projects.title",
        ),
        (
            "project_detail.html",
            include_str!("../src/web/project_detail.html"),
            "detail.open_dir",
        ),
        (
            "dependencies.html",
            include_str!("../src/web/dependencies.html"),
            "deps.title",
        ),
    ];
    for (name, html, sample_key) in &pages {
        assert!(html.contains("data-i18n"), "{} 应包含 data-i18n 属性", name);
        assert!(
            html.contains(sample_key),
            "{} 应包含翻译 key {}",
            name,
            sample_key
        );
    }
}

#[test]
fn i18n_module_has_locale_detection() {
    let js = include_str!("../src/web/i18n.js");
    assert!(
        js.contains("pyforge_locale"),
        "i18n.js 应使用 pyforge_locale localStorage key"
    );
    assert!(js.contains("getLocale"), "i18n.js 应包含 getLocale 函数");
    assert!(
        js.contains("navigator.language"),
        "i18n.js 应检测 navigator.language"
    );
}

#[test]
fn i18n_module_has_translations() {
    let js = include_str!("../src/web/i18n.js");
    assert!(js.contains("'zh'"), "i18n.js 应包含中文翻译");
    assert!(js.contains("'en'"), "i18n.js 应包含英文翻译");
    assert!(js.contains("applyI18n"), "i18n.js 应包含 applyI18n 函数");
    assert!(
        js.contains("localeChanged"),
        "i18n.js 应触发 localeChanged 事件"
    );
}

#[test]
fn i18n_module_exports_api() {
    let js = include_str!("../src/web/i18n.js");
    assert!(
        js.contains("PyForgeI18n"),
        "i18n.js 应导出 PyForgeI18n 全局对象"
    );
    assert!(js.contains("getLocale:"), "i18n.js 应导出 getLocale");
    assert!(js.contains("t:"), "i18n.js 应导出 t 翻译函数");
    assert!(js.contains("applyI18n:"), "i18n.js 应导出 applyI18n");
    assert!(js.contains("initI18n:"), "i18n.js 应导出 initI18n");
}

#[test]
fn i18n_server_route_exists() {
    let server = include_str!("../src/web/server.rs");
    assert!(server.contains("i18n.js"), "server.rs 应包含 i18n.js 路由");
    assert!(
        server.contains("application/javascript"),
        "server.rs 应设置正确的 Content-Type"
    );
}
