//! 索引模块：IndexStore trait + 数据类型 + 错误类型。
//!
//! 本模块为 Epic 0 基础设施层，后续 CLI 命令通过 `IndexStore` trait 操作索引，
//! 无需感知底层存储（JSON / 未来 SQLite）。
//!
//! `#[allow(dead_code)]` 是因为本 story 仅定义接口与实现，
//! 真正的调用方在 Epic 1 (track/list/scan 等) 中。

pub mod types;
pub mod store;
pub mod error;
pub mod detector;
pub mod scanner;
