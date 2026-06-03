//! 索引数据类型定义。
//!
//! [Source: architecture.md §Naming Patterns]
//! - 所有公开类型派生 `Debug, Clone`
//! - JSON 序列化类型派生 `Serialize, Deserialize`
//! - JSON 字段使用 `snake_case`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 单个项目的元数据记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub python_version: Option<String>,
    pub toolchain: Option<String>,
    pub git_remote: Option<String>,
    pub git_branch: Option<String>,
    pub git_status: Option<String>,
    pub deps_count: Option<u64>,
    pub created_at: Option<String>,
    pub last_modified: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

/// 索引文件顶层结构。
///
/// [Source: prd.md §8] `version` 字段用于向前兼容。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexData {
    pub version: String,
    pub projects: HashMap<String, ProjectInfo>,
}

impl IndexData {
    /// 返回带有默认 version 字段的空索引。
    pub fn empty() -> Self {
        Self {
            version: "1.0".to_string(),
            projects: HashMap::new(),
        }
    }
}
