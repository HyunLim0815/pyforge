//! JSON 输出格式工具。
//!
//! [Source: architecture.md §JSON Output Format]
//! - 成功: `{ "version": "1.0", "success": true, "data": {...} }`
//! - 失败: `{ "version": "1.0", "success": false, "error": {"code": "...", "namespace": "...", "message": "..."} }`

use serde::Serialize;
use serde_json::Value;

const VERSION: &str = "1.0";

#[derive(Serialize)]
struct JsonEnvelope<T: Serialize> {
    version: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonErrorBody>,
}

#[derive(Serialize)]
struct JsonErrorBody {
    code: String,
    namespace: String,
    message: String,
}

/// 构建成功 JSON 并序列化为字符串。
pub fn success<T: Serialize>(data: T) -> String {
    let envelope = JsonEnvelope {
        version: VERSION.into(),
        success: true,
        data: Some(data),
        error: None,
    };
    serde_json::to_string(&envelope).unwrap_or_else(|e| {
        let safe_msg = serde_json::to_string(&e.to_string()).unwrap_or_else(|_| r#""serialization error""#.into());
        format!(r#"{{"version":"{}","success":false,"error":{{"code":"SERIALIZATION_ERROR","namespace":"CLI","message":{}}}}}"#, VERSION, safe_msg)
    })
}

/// 构建错误 JSON 并序列化为字符串。
pub fn error(namespace: &str, code: &str, message: &str) -> String {
    let envelope = JsonEnvelope::<Value> {
        version: VERSION.into(),
        success: false,
        data: None,
        error: Some(JsonErrorBody {
            code: code.into(),
            namespace: namespace.into(),
            message: message.into(),
        }),
    };
    serde_json::to_string(&envelope).unwrap_or_else(|_| {
        format!(r#"{{"version":"{}","success":false,"error":{{"code":"SERIALIZATION_ERROR","namespace":"CLI","message":"json serialize failed"}}}}"#, VERSION)
    })
}
