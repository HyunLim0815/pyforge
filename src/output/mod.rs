//! 输出格式模块：Plain（人类可读）与 Json（结构化）双模式。

pub mod error;
pub mod json;

/// 输出格式枚举，由全局 `--json` flag 驱动。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum OutputFormat {
    Plain,
    Json,
}
