//! uv 桥接错误。

use std::fmt;

#[derive(Debug)]
pub enum UvError {
    NotInstalled,
    InitFailed(String),
    ParseError(String),
    Io(std::io::Error),
}

impl fmt::Display for UvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInstalled => write!(f, "uv 未安装"),
            Self::InitFailed(msg) => write!(f, "uv init 失败: {}", msg),
            Self::ParseError(msg) => write!(f, "uv 输出解析失败: {}", msg),
            Self::Io(e) => write!(f, "uv 执行错误: {}", e),
        }
    }
}

impl std::error::Error for UvError {}

impl From<std::io::Error> for UvError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::NotFound {
            Self::NotInstalled
        } else {
            Self::Io(e)
        }
    }
}
