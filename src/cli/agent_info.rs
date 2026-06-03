//! `pyforge agent-info` 命令实现。
//!
//! 输出 AI Agent 环境上下文摘要，支持 `--json` 全局标志。

use super::Cli;
use crate::index::store::{IndexStore, JsonStore};
use crate::output::json;
use crate::templates::BUILTIN_TEMPLATES;
use crate::uv::bridge;
use std::path::PathBuf;

/// agent-info 数据结构，对应 PRD §5.3。
#[derive(serde::Serialize, Clone, Debug)]
struct AgentInfoData {
    pyforge_version: String,
    uv_installed: bool,
    uv_version: Option<String>,
    total_projects: usize,
    index_path: String,
    templates_available: Vec<&'static str>,
    recent_projects: Vec<RecentProject>,
    system: SystemInfo,
}

#[derive(serde::Serialize, Clone, Debug)]
struct RecentProject {
    name: String,
    path: String,
    last_modified: Option<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
struct SystemInfo {
    os: String,
    shell: String,
    python_default: Option<String>,
}

pub fn handle(cli: &Cli) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd::agent-info").entered();

    let data = collect_info().map_err(|e| {
        eprintln!("{}", e);
        2
    })?;

    if cli.json {
        println!("{}", json::success(&data));
    } else {
        print_plain(&data);
    }

    Ok(())
}

fn collect_info() -> Result<AgentInfoData, Box<dyn std::error::Error>> {
    // 1. pyforge 版本
    let pyforge_version = env!("CARGO_PKG_VERSION").to_string();

    // 2. uv 信息
    let uv_info = bridge::version();

    // 3. 索引路径与项目列表
    let index_path = std::env::var("PYFORGE_INDEX_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| JsonStore::default_path());
    let store = JsonStore::open(index_path.clone())?;
    let all_projects = store.list_projects();
    let total_projects = all_projects.len();

    // 4. 最近项目（按 last_modified desc 取 5）
    let mut recent_projects: Vec<RecentProject> = all_projects
        .into_iter()
        .map(|p| RecentProject {
            name: p.name,
            path: p.path,
            last_modified: p.last_modified,
        })
        .collect();
    recent_projects.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    recent_projects.truncate(5);

    // 5. 可用模板
    let templates_available: Vec<&'static str> = BUILTIN_TEMPLATES.to_vec();

    // 6. 系统信息
    let system = collect_system_info();

    Ok(AgentInfoData {
        pyforge_version,
        uv_installed: uv_info.installed,
        uv_version: uv_info.version,
        total_projects,
        index_path: index_path.display().to_string(),
        templates_available,
        recent_projects,
        system,
    })
}

fn collect_system_info() -> SystemInfo {
    let os = std::env::consts::OS.to_string();

    let shell = std::env::var("SHELL")
        .or_else(|_| std::env::var("COMSPEC"))
        .unwrap_or_else(|_| "unknown".into());
    // 取 shell 路径的文件名
    let shell = std::path::Path::new(&shell)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or(shell);

    let python_default = find_python_version();

    SystemInfo {
        os,
        shell,
        python_default,
    }
}

/// 尝试获取默认 Python 版本。
fn find_python_version() -> Option<String> {
    let python_bin = if cfg!(windows) { "python" } else { "python3" };
    std::process::Command::new(python_bin)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            // "Python 3.12.4" → "3.12.4"
            stdout
                .strip_prefix("Python ")
                .unwrap_or(&stdout)
                .to_string()
        })
}

fn print_plain(data: &AgentInfoData) {
    println!("pyforge {}", data.pyforge_version);
    let uv_status = if data.uv_installed {
        data.uv_version.as_deref().unwrap_or("unknown")
    } else {
        "not installed"
    };
    println!("uv: {}", uv_status);
    println!("projects: {}", data.total_projects);
    println!("index: {}", data.index_path);
    println!("templates: {}", data.templates_available.join(", "));
    println!(
        "system: {} / {} / python {}",
        data.system.os,
        data.system.shell,
        data.system.python_default.as_deref().unwrap_or("N/A")
    );
    if !data.recent_projects.is_empty() {
        println!("recent:");
        for p in &data.recent_projects {
            let ts = p.last_modified.as_deref().unwrap_or("-");
            println!("  {} ({})", p.name, ts);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // 串行化 PYFORGE_UV_MOCK 环境变量测试，避免并发竞争
    static UV_MOCK_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn collect_system_info_returns_valid_os() {
        let info = collect_system_info();
        assert!(!info.os.is_empty());
        assert!(!info.shell.is_empty());
    }

    #[test]
    fn agent_info_json_structure_with_mock() {
        let _guard = UV_MOCK_LOCK.lock().unwrap();

        std::env::set_var("PYFORGE_UV_MOCK", "success");
        let tmp = tempfile::TempDir::new().unwrap();
        let index_path = tmp.path().join("projects.json");
        std::env::set_var("PYFORGE_INDEX_PATH", &index_path);

        let result = collect_info();
        std::env::remove_var("PYFORGE_UV_MOCK");
        std::env::remove_var("PYFORGE_INDEX_PATH");

        let data = result.unwrap();
        assert_eq!(data.pyforge_version, env!("CARGO_PKG_VERSION"));
        assert!(data.uv_installed);
        assert!(data.uv_version.is_some());
        assert_eq!(data.total_projects, 0);
        assert_eq!(data.templates_available, vec!["fastapi", "cli", "lib"]);
        assert!(data.recent_projects.is_empty());

        // 验证 JSON 序列化符合统一结构
        let json_str = json::success(&data);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["version"], "1.0");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["pyforge_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(parsed["data"]["uv_installed"], true);
        assert!(parsed["data"]["system"]["os"].is_string());
    }

    #[test]
    fn agent_info_uv_not_installed() {
        let _guard = UV_MOCK_LOCK.lock().unwrap();

        std::env::set_var("PYFORGE_UV_MOCK", "missing");
        let tmp = tempfile::TempDir::new().unwrap();
        let index_path = tmp.path().join("projects.json");
        std::env::set_var("PYFORGE_INDEX_PATH", &index_path);

        let result = collect_info();
        std::env::remove_var("PYFORGE_UV_MOCK");
        std::env::remove_var("PYFORGE_INDEX_PATH");

        let data = result.unwrap();
        assert!(!data.uv_installed);
        assert!(data.uv_version.is_none());
    }

    #[test]
    fn recent_projects_sorted_by_last_modified() {
        use crate::index::types::ProjectInfo;

        let _guard = UV_MOCK_LOCK.lock().unwrap();

        let tmp = tempfile::TempDir::new().unwrap();
        let index_path = tmp.path().join("projects.json");
        let mut store = JsonStore::open(index_path.clone()).unwrap();

        store
            .add_project(ProjectInfo {
                name: "old".into(),
                path: "/tmp/old".into(),
                python_version: None,
                toolchain: None,
                git_remote: None,
                git_branch: None,
                git_status: None,
                deps_count: None,
                created_at: None,
                last_modified: Some("2025-01-01".into()),
                tags: vec![],
                description: None,
            })
            .unwrap();
        store
            .add_project(ProjectInfo {
                name: "new".into(),
                path: "/tmp/new".into(),
                python_version: None,
                toolchain: None,
                git_remote: None,
                git_branch: None,
                git_status: None,
                deps_count: None,
                created_at: None,
                last_modified: Some("2025-06-01".into()),
                tags: vec![],
                description: None,
            })
            .unwrap();
        drop(store);

        std::env::set_var("PYFORGE_UV_MOCK", "missing");
        std::env::set_var("PYFORGE_INDEX_PATH", &index_path);

        let data = collect_info().unwrap();
        std::env::remove_var("PYFORGE_UV_MOCK");
        std::env::remove_var("PYFORGE_INDEX_PATH");

        assert_eq!(data.total_projects, 2);
        assert_eq!(data.recent_projects.len(), 2);
        assert_eq!(data.recent_projects[0].name, "new");
        assert_eq!(data.recent_projects[1].name, "old");
    }

    #[test]
    fn recent_projects_truncated_to_5() {
        use crate::index::types::ProjectInfo;

        let _guard = UV_MOCK_LOCK.lock().unwrap();

        let tmp = tempfile::TempDir::new().unwrap();
        let index_path = tmp.path().join("projects.json");
        let mut store = JsonStore::open(index_path.clone()).unwrap();

        for i in 0..8 {
            store
                .add_project(ProjectInfo {
                    name: format!("p{}", i),
                    path: format!("/tmp/p{}", i),
                    python_version: None,
                    toolchain: None,
                    git_remote: None,
                    git_branch: None,
                    git_status: None,
                    deps_count: None,
                    created_at: None,
                    last_modified: Some(format!("2025-0{}", i + 1)),
                    tags: vec![],
                    description: None,
                })
                .unwrap();
        }
        drop(store);

        std::env::set_var("PYFORGE_UV_MOCK", "missing");
        std::env::set_var("PYFORGE_INDEX_PATH", &index_path);

        let data = collect_info().unwrap();
        std::env::remove_var("PYFORGE_UV_MOCK");
        std::env::remove_var("PYFORGE_INDEX_PATH");

        assert_eq!(data.total_projects, 8);
        assert_eq!(data.recent_projects.len(), 5);
    }
}
