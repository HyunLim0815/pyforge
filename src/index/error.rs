//! 索引层错误类型。
//!
//! [Source: architecture.md §JSON Output Format]
//! 错误码命名空间使用 `INDEX_*`，后续由 CLI 层映射为 JSON 错误对象。

use std::fmt;

/// 索引操作错误。
#[derive(Debug)]
pub enum IndexError {
    /// 底层存储错误（IO）。
    StoreError(std::io::Error),
    /// 项目不在索引中。
    ProjectNotFound(String),
    /// 项目已存在索引中。
    AlreadyTracked(String),
    /// JSON 序列化/反序列化错误。
    SerializationError(serde_json::Error),
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StoreError(e) => write!(f, "存储错误: {}", e),
            Self::ProjectNotFound(name) => write!(f, "项目 '{}' 不在索引中", name),
            Self::AlreadyTracked(name) => write!(f, "项目 '{}' 已在索引中", name),
            Self::SerializationError(e) => write!(f, "序列化错误: {}", e),
        }
    }
}

impl std::error::Error for IndexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::StoreError(e) => Some(e),
            Self::SerializationError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for IndexError {
    fn from(e: std::io::Error) -> Self {
        Self::StoreError(e)
    }
}

impl From<serde_json::Error> for IndexError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerializationError(e)
    }
}
