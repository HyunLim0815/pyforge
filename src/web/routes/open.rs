//! 一键操作 API。
//!
//! `POST /api/open` 打开项目目录、终端或 IDE。

use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::index::store::{IndexStore, JsonStore};
use crate::system::open::{self, OpenAction};

/// 请求体。
#[derive(Debug, Deserialize)]
pub struct OpenRequest {
    /// 操作类型：dir | terminal | vscode。
    pub action: String,
    /// 项目名称。
    pub project: String,
}

/// 成功响应。
#[derive(Debug, Serialize)]
pub struct OpenResponse {
    pub success: bool,
    pub message: String,
}

/// 错误响应。
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

/// `POST /api/open` — 打开项目目录/终端/IDE。
pub async fn post_open(
    Json(req): Json<OpenRequest>,
) -> Result<Json<OpenResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 1. 白名单校验 action
    let action = OpenAction::from_str(&req.action).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: format!(
                    "不支持的操作: '{}'。允许的值: dir, terminal, vscode",
                    req.action
                ),
            }),
        )
    })?;

    // 2. 校验项目名（防注入：仅允许字母数字下划线连字符点）
    if !is_valid_project_name(&req.project) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "项目名称包含非法字符".into(),
            }),
        ));
    }

    // 3. 查找项目（仅允许操作索引内项目）
    let store = JsonStore::load_or_create().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                success: false,
                error: format!("加载索引失败: {}", e),
            }),
        )
    })?;

    let project = store.get_project(&req.project).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                success: false,
                error: format!("项目不存在: {}", req.project),
            }),
        )
    })?;

    // 4. 验证项目路径存在且为目录
    let path = std::path::Path::new(&project.path);
    if !path.is_dir() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                success: false,
                error: format!("项目目录不存在: {}", project.path),
            }),
        ));
    }

    // 5. 执行操作
    open::execute(action, path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                success: false,
                error: e,
            }),
        )
    })?;

    Ok(Json(OpenResponse {
        success: true,
        message: format!("已打开 {}: {}", req.action, project.path),
    }))
}

/// 校验项目名称：仅允许字母、数字、下划线、连字符、点。
fn is_valid_project_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 256
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_project_names() {
        assert!(is_valid_project_name("my-project"));
        assert!(is_valid_project_name("fastapi_app"));
        assert!(is_valid_project_name("project.v2"));
        assert!(is_valid_project_name("a"));
    }

    #[test]
    fn invalid_project_names() {
        assert!(!is_valid_project_name(""));
        assert!(!is_valid_project_name("../etc/passwd"));
        assert!(!is_valid_project_name("name; rm -rf /"));
        assert!(!is_valid_project_name("name$(whoami)"));
        assert!(!is_valid_project_name("name|pipe"));
        assert!(!is_valid_project_name("name with spaces"));
        assert!(!is_valid_project_name("name/slash"));
    }

    #[test]
    fn project_name_length_limit() {
        let long_name = "a".repeat(257);
        assert!(!is_valid_project_name(&long_name));
        let max_name = "a".repeat(256);
        assert!(is_valid_project_name(&max_name));
    }
}
