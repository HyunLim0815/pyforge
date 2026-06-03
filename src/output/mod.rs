//! 输出格式模块：Plain（人类可读）与 Json（结构化）双模式。

pub mod json;
pub mod error;

/// 输出格式枚举，由全局 `--json` flag 驱动。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum OutputFormat {
    Plain,
    Json,
}
