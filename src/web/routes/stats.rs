//! 仪表盘统计 API。
//!
//! `GET /api/stats` 返回项目统计数据。

use axum::Json;
use serde::Serialize;
use std::collections::HashMap;

use crate::index::store::{IndexStore, JsonStore};

/// 单个活跃项目摘要。
#[derive(Debug, Clone, Serialize)]
pub struct ActiveProject {
    pub name: String,
    pub path: String,
    pub python_version: Option<String>,
    pub toolchain: Option<String>,
    pub last_modified: Option<String>,
}

/// `/api/stats` 响应体。
#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    pub total: usize,
    pub by_python_version: HashMap<String, usize>,
    pub by_toolchain: HashMap<String, usize>,
    pub recent_active: Vec<ActiveProject>,
}

/// `GET /api/stats` — 返回项目统计 JSON。
pub async fn get_stats() -> Json<Stats> {
    let store = match JsonStore::load_or_create() {
        Ok(s) => s,
        Err(_) => {
            return Json(Stats {
                total: 0,
                by_python_version: HashMap::new(),
                by_toolchain: HashMap::new(),
                recent_active: Vec::new(),
            })
        }
    };
    let projects = store.list_projects();

    let mut by_python_version: HashMap<String, usize> = HashMap::new();
    let mut by_toolchain: HashMap<String, usize> = HashMap::new();
    let mut recent_active: Vec<ActiveProject> = Vec::new();

    let now = chrono_or_now();

    for p in &projects {
        // Python 版本分布
        let ver = p.python_version.clone().unwrap_or_else(|| "unknown".into());
        *by_python_version.entry(ver).or_insert(0) += 1;

        // 工具链分布
        let tool = p.toolchain.clone().unwrap_or_else(|| "unknown".into());
        *by_toolchain.entry(tool).or_insert(0) += 1;

        // 活跃项目（7 天内修改）
        if let Some(ts) = &p.last_modified {
            if is_recent(ts, &now, 7) {
                recent_active.push(ActiveProject {
                    name: p.name.clone(),
                    path: p.path.clone(),
                    python_version: p.python_version.clone(),
                    toolchain: p.toolchain.clone(),
                    last_modified: p.last_modified.clone(),
                });
            }
        }
    }

    // 按 last_modified 降序排列
    recent_active.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    Json(Stats {
        total: projects.len(),
        by_python_version,
        by_toolchain,
        recent_active,
    })
}

/// 获取当前时间戳字符串（ISO 8601 格式）。
fn chrono_or_now() -> String {
    // 使用简单的时间戳格式，避免引入 chrono 依赖
    // 格式：YYYY-MM-DD HH:MM:SS 或 ISO 8601
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // 返回 Unix 时间戳秒数，用于比较
    now.to_string()
}

/// 判断时间戳是否在最近 N 天内。
///
/// 支持的时间戳格式：
/// - Unix 时间戳（秒）
/// - ISO 8601 格式（如 "2026-01-15T10:30:00Z"）
/// - 简单日期格式（如 "2026-01-15"）
fn is_recent(ts: &str, now: &str, days: u64) -> bool {
    let seconds_in_day: u64 = 86400;
    let threshold = days * seconds_in_day;

    // 尝试解析为 Unix 时间戳
    if let (Ok(ts_val), Ok(now_val)) = (ts.parse::<u64>(), now.parse::<u64>()) {
        return now_val.saturating_sub(ts_val) <= threshold;
    }

    // 尝试解析 ISO 8601 格式
    if let (Some(ts_secs), Some(now_secs)) = (parse_iso8601_to_secs(ts), now.parse::<u64>().ok()) {
        return now_secs.saturating_sub(ts_secs) <= threshold;
    }

    false
}

/// 简单解析 ISO 8601 日期时间到 Unix 时间戳秒数。
///
/// 仅支持 "YYYY-MM-DD" 和 "YYYY-MM-DDTHH:MM:SS" 格式。
fn parse_iso8601_to_secs(s: &str) -> Option<u64> {
    // 简单解析 YYYY-MM-DD 部分
    let date_part = if s.len() >= 10 { &s[..10] } else { s };
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: u64 = parts[0].parse().ok()?;
    let month: u64 = parts[1].parse().ok()?;
    let day: u64 = parts[2].parse().ok()?;

    // 简化的日期转时间戳（不考虑时区和闰秒）
    let days_since_epoch = days_since_epoch(year, month, day)?;
    Some(days_since_epoch * 86400)
}

/// 计算从 Unix 纪元（1970-01-01）到指定日期的天数。
fn days_since_epoch(year: u64, month: u64, day: u64) -> Option<u64> {
    if month == 0 || month > 12 || day == 0 || day > 31 {
        return None;
    }

    let mut days: u64 = 0;

    // 年份天数
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    // 月份天数
    let month_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        days += month_days[m as usize];
        if m == 2 && is_leap_year(year) {
            days += 1;
        }
    }

    days += day - 1;
    Some(days)
}

fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_json_structure() {
        // 验证 Stats 结构体序列化为正确的 JSON 格式
        let stats = Stats {
            total: 5,
            by_python_version: HashMap::from([("3.12".into(), 3), ("3.11".into(), 2)]),
            by_toolchain: HashMap::from([("uv".into(), 4), ("pip".into(), 1)]),
            recent_active: vec![ActiveProject {
                name: "test-project".into(),
                path: "/tmp/test".into(),
                python_version: Some("3.12".into()),
                toolchain: Some("uv".into()),
                last_modified: Some("2026-06-01".into()),
            }],
        };

        let json = serde_json::to_value(&stats).unwrap();
        assert_eq!(json["total"], 5);
        assert!(json["by_python_version"].is_object());
        assert!(json["by_toolchain"].is_object());
        assert!(json["recent_active"].is_array());
        assert_eq!(json["recent_active"][0]["name"], "test-project");
    }

    #[test]
    fn is_recent_with_unix_timestamp() {
        let now = "1000000000"; // 某个时间点
        let old = "999000000"; // ~11.6 天前
        let very_recent = "999999900"; // ~100 秒前

        assert!(!is_recent(old, now, 7));
        assert!(is_recent(very_recent, now, 7));
    }

    #[test]
    fn is_recent_with_iso8601() {
        // 使用 Unix 时间戳作为 now
        // 2026-01-01 = 约 20454 天 * 86400 = 1767225600 秒
        let now = "1767225600"; // 约 2026-01-01
        let recent = "2025-12-30"; // 2 天前
        let old = "2025-12-01"; // 31 天前

        assert!(is_recent(recent, now, 7));
        assert!(!is_recent(old, now, 7));
    }

    #[test]
    fn parse_iso8601_date() {
        let secs = parse_iso8601_to_secs("2026-01-15").unwrap();
        // 2026-01-15 应该是一个合理的时间戳
        assert!(secs > 1_700_000_000);
    }

    #[test]
    fn days_since_epoch_basic() {
        // 1970-01-01 = 0
        assert_eq!(days_since_epoch(1970, 1, 1), Some(0));
        // 1970-01-02 = 1
        assert_eq!(days_since_epoch(1970, 1, 2), Some(1));
        // 1971-01-01 = 365
        assert_eq!(days_since_epoch(1971, 1, 1), Some(365));
    }

    #[test]
    fn is_leap_year_check() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
    }
}
