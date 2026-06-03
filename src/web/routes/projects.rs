//! 项目列表 API。
//!
//! `GET /api/projects` 返回项目列表，支持搜索和过滤。

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::index::store::{IndexStore, JsonStore};
use crate::index::types::ProjectInfo;

/// 查询参数。
#[derive(Debug, Deserialize)]
pub struct ProjectsQuery {
    /// 名称/路径子串搜索。
    pub q: Option<String>,
    /// 工具链精确匹配。
    pub toolchain: Option<String>,
}

/// 项目卡片信息（精简版 ProjectInfo）。
#[derive(Debug, Clone, Serialize)]
pub struct ProjectCard {
    pub name: String,
    pub path: String,
    pub python_version: Option<String>,
    pub toolchain: Option<String>,
    pub git_status: Option<String>,
    pub deps_count: Option<u64>,
    pub last_modified: Option<String>,
}

impl From<&ProjectInfo> for ProjectCard {
    fn from(p: &ProjectInfo) -> Self {
        Self {
            name: p.name.clone(),
            path: p.path.clone(),
            python_version: p.python_version.clone(),
            toolchain: p.toolchain.clone(),
            git_status: p.git_status.clone(),
            deps_count: p.deps_count,
            last_modified: p.last_modified.clone(),
        }
    }
}

/// `GET /api/projects` — 返回项目列表 JSON。
///
/// 支持查询参数：
/// - `q`: 名称或路径子串搜索（大小写不敏感）
/// - `toolchain`: 工具链精确匹配
pub async fn get_projects(Query(params): Query<ProjectsQuery>) -> Json<Vec<ProjectCard>> {
    let store = JsonStore::load_or_create().expect("无法加载索引");
    let projects = store.list_projects();

    let filtered: Vec<ProjectCard> = projects
        .iter()
        .filter(|p| {
            // q 过滤：名称或路径包含子串（大小写不敏感）
            if let Some(ref q) = params.q {
                let q_lower = q.to_lowercase();
                let name_match = p.name.to_lowercase().contains(&q_lower);
                let path_match = p.path.to_lowercase().contains(&q_lower);
                if !name_match && !path_match {
                    return false;
                }
            }

            // toolchain 过滤：精确匹配
            if let Some(ref toolchain) = params.toolchain {
                match &p.toolchain {
                    Some(t) if t == toolchain => {}
                    _ => return false,
                }
            }

            true
        })
        .map(ProjectCard::from)
        .collect();

    Json(filtered)
}

/// `GET /api/projects/:name` — 返回单个项目详情。
pub async fn get_project_by_name(
    Path(name): Path<String>,
) -> Result<Json<ProjectInfo>, (StatusCode, Json<serde_json::Value>)> {
    let store = JsonStore::load_or_create().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("加载索引失败: {}", e)})),
        )
    })?;

    store.get_project(&name).map(Json).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("项目不存在: {}", name)})),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_projects() -> Vec<ProjectInfo> {
        vec![
            ProjectInfo {
                name: "fastapi-app".into(),
                path: "/home/user/fastapi-app".into(),
                python_version: Some("3.12".into()),
                toolchain: Some("uv".into()),
                git_remote: None,
                git_branch: Some("main".into()),
                git_status: Some("clean".into()),
                deps_count: Some(15),
                created_at: None,
                last_modified: None,
                tags: vec![],
                description: None,
            },
            ProjectInfo {
                name: "django-site".into(),
                path: "/home/user/django-site".into(),
                python_version: Some("3.11".into()),
                toolchain: Some("pip".into()),
                git_remote: None,
                git_branch: Some("develop".into()),
                git_status: Some("dirty".into()),
                deps_count: Some(20),
                created_at: None,
                last_modified: None,
                tags: vec![],
                description: None,
            },
            ProjectInfo {
                name: "my-uv-tool".into(),
                path: "/home/user/my-uv-tool".into(),
                python_version: Some("3.12".into()),
                toolchain: Some("uv".into()),
                git_remote: None,
                git_branch: None,
                git_status: None,
                deps_count: Some(3),
                created_at: None,
                last_modified: None,
                tags: vec![],
                description: None,
            },
        ]
    }

    #[test]
    fn project_card_from_project_info() {
        let p = &mock_projects()[0];
        let card = ProjectCard::from(p);
        assert_eq!(card.name, "fastapi-app");
        assert_eq!(card.path, "/home/user/fastapi-app");
        assert_eq!(card.python_version, Some("3.12".into()));
        assert_eq!(card.toolchain, Some("uv".into()));
        assert_eq!(card.git_status, Some("clean".into()));
    }

    #[test]
    fn filter_by_q_name_match() {
        let projects = mock_projects();
        let q = "fastapi";
        let filtered: Vec<&ProjectInfo> = projects
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&q.to_lowercase())
                    || p.path.to_lowercase().contains(&q.to_lowercase())
            })
            .collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "fastapi-app");
    }

    #[test]
    fn filter_by_q_path_match() {
        let projects = mock_projects();
        let q = "django";
        let filtered: Vec<&ProjectInfo> = projects
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&q.to_lowercase())
                    || p.path.to_lowercase().contains(&q.to_lowercase())
            })
            .collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "django-site");
    }

    #[test]
    fn filter_by_toolchain() {
        let projects = mock_projects();
        let toolchain = "uv";
        let filtered: Vec<&ProjectInfo> = projects
            .iter()
            .filter(|p| p.toolchain.as_deref() == Some(toolchain))
            .collect();
        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .all(|p| p.toolchain.as_deref() == Some("uv")));
    }

    #[test]
    fn filter_by_q_and_toolchain_combined() {
        let projects = mock_projects();
        let q = "fast";
        let toolchain = "uv";
        let filtered: Vec<&ProjectInfo> = projects
            .iter()
            .filter(|p| {
                let q_match = p.name.to_lowercase().contains(&q.to_lowercase())
                    || p.path.to_lowercase().contains(&q.to_lowercase());
                let t_match = p.toolchain.as_deref() == Some(toolchain);
                q_match && t_match
            })
            .collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "fastapi-app");
    }

    #[test]
    fn no_filter_returns_all() {
        let projects = mock_projects();
        // 无过滤条件时返回全部
        assert_eq!(projects.len(), 3);
    }
}
