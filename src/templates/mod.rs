//! 模板渲染模块。
//!
//! 内置模板编译进二进制；自定义模板从 `PYFORGE_TEMPLATE_DIR/<name>` 或数据目录读取。

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum TemplateError {
    UnknownTemplate(String),
    Io(std::io::Error),
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTemplate(t) => write!(f, "未知模板: {}。可用模板: fastapi, cli, lib", t),
            Self::Io(e) => write!(f, "模板渲染失败: {}", e),
        }
    }
}

impl std::error::Error for TemplateError {}

impl From<std::io::Error> for TemplateError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// 内置模板名称列表。
pub const BUILTIN_TEMPLATES: &[&str] = &["fastapi", "cli", "lib"];

pub fn render(template: &str, project_name: &str, dest: &Path) -> Result<(), TemplateError> {
    match template {
        "fastapi" => {
            warn_if_custom_shadowed(template);
            render_builtin(project_name, dest, FASTAPI_FILES)
        }
        "cli" => {
            warn_if_custom_shadowed(template);
            render_builtin(project_name, dest, CLI_FILES)
        }
        "lib" => {
            warn_if_custom_shadowed(template);
            render_builtin(project_name, dest, LIB_FILES)
        }
        other => render_custom(other, project_name, dest),
    }
}

/// 若自定义模板目录存在同名模板，输出警告（内置优先）。
fn warn_if_custom_shadowed(template: &str) {
    let custom_root = custom_template_root().join(template);
    if custom_root.exists() {
        eprintln!("{}", crate::t!("template.custom_shadowed", template));
    }
}

fn render_builtin(
    project_name: &str,
    dest: &Path,
    files: &[(&str, &str)],
) -> Result<(), TemplateError> {
    for (rel, content) in files {
        let rendered_rel = apply_vars(rel, project_name);
        let rendered_content = apply_vars(content, project_name);
        let path = dest.join(rendered_rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, rendered_content)?;
    }
    Ok(())
}

fn render_custom(template: &str, project_name: &str, dest: &Path) -> Result<(), TemplateError> {
    let root = custom_template_root().join(template);
    if !root.exists() {
        return Err(TemplateError::UnknownTemplate(template.into()));
    }
    copy_dir_with_vars(&root, dest, project_name)
}

fn custom_template_root() -> PathBuf {
    std::env::var("PYFORGE_TEMPLATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("pyforge")
                .join("templates")
        })
}

fn copy_dir_with_vars(src: &Path, dest: &Path, project_name: &str) -> Result<(), TemplateError> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let rel_name = apply_vars(&entry.file_name().to_string_lossy(), project_name);
        let target = dest.join(rel_name);
        if path.is_dir() {
            fs::create_dir_all(&target)?;
            copy_dir_with_vars(&path, &target, project_name)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            // 检测二进制文件：尝试读取为 UTF-8，失败则直接复制
            match fs::read_to_string(&path) {
                Ok(content) => {
                    fs::write(target, apply_vars(&content, project_name))?;
                }
                Err(_) => {
                    // 二进制文件直接复制，不做变量替换
                    fs::copy(&path, &target)?;
                }
            }
        }
    }
    Ok(())
}

fn apply_vars(input: &str, project_name: &str) -> String {
    input.replace("{{name}}", project_name)
}

const FASTAPI_FILES: &[(&str, &str)] = &[
    (
        "app/main.py",
        include_str!("../../templates/fastapi/app/main.py"),
    ),
    (
        "app/routers/__init__.py",
        include_str!("../../templates/fastapi/app/routers/__init__.py"),
    ),
    (
        "app/models/__init__.py",
        include_str!("../../templates/fastapi/app/models/__init__.py"),
    ),
    (
        "tests/__init__.py",
        include_str!("../../templates/fastapi/tests/__init__.py"),
    ),
];

const CLI_FILES: &[(&str, &str)] = &[(
    "src/main.py",
    include_str!("../../templates/cli/src/main.py"),
)];

const LIB_FILES: &[(&str, &str)] = &[
    (
        "src/__init__.py",
        include_str!("../../templates/lib/src/__init__.py"),
    ),
    (
        "src/{{name}}.py",
        include_str!("../../templates/lib/src/{{name}}.py"),
    ),
];
