//! 索引存储 trait 与 JSON 实现。
//!
//! [Source: architecture.md §Decision Priority Analysis]
//! - `IndexStore` trait 抽象存储后端，v0.1 用 JSON，未来可切换 SQLite
//! - JSON 写入策略：`.tmp` → `std::fs::rename` 原子替换
//! - 路径解析使用 `dirs::data_dir().join("pyforge").join("projects.json")`

use crate::index::error::IndexError;
use crate::index::types::{IndexData, ProjectInfo};
use fs2::FileExt;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// 索引存储抽象 trait。
///
/// 所有 CLI 命令通过此 trait 操作索引，不直接读写 JSON。
pub trait IndexStore {
    fn read(&self) -> Result<IndexData, IndexError>;
    fn write(&self, data: &IndexData) -> Result<(), IndexError>;
    fn add_project(&mut self, info: ProjectInfo) -> Result<(), IndexError>;
    fn remove_project(&mut self, name: &str) -> Result<(), IndexError>;
    fn get_project(&self, name: &str) -> Option<ProjectInfo>;
    fn list_projects(&self) -> Vec<ProjectInfo>;
}

/// JSON 文件索引存储实现。
pub struct JsonStore {
    path: PathBuf,
    /// 内存缓存，避免每次操作都读盘。
    cache: IndexData,
}

impl JsonStore {
    /// 创建 JsonStore 实例，指向默认索引路径。
    ///
    /// 等价于 `dirs::data_dir().join("pyforge").join("projects.json")`。
    pub fn default_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pyforge")
            .join("projects.json")
    }

    /// 从默认路径加载或创建空索引。
    pub fn load_or_create() -> Result<Self, IndexError> {
        Self::open(Self::default_path())
    }

    /// 从指定路径打开索引文件。目录不存在时自动创建。
    /// 文件不存在时返回空 IndexData。
    pub fn open(path: PathBuf) -> Result<Self, IndexError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let cache = Self::read_file(&path)?;
        Ok(Self { path, cache })
    }

    /// 内部读取 JSON 文件；文件不存在返回空索引。
    fn read_file(path: &Path) -> Result<IndexData, IndexError> {
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(IndexData::empty()),
            Err(e) => return Err(IndexError::StoreError(e)),
        };
        // 限制最大 10MB，防止损坏文件耗尽内存
        let metadata = file.metadata()?;
        if metadata.len() > 10 * 1024 * 1024 {
            return Err(IndexError::StoreError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "索引文件过大，可能已损坏",
            )));
        }
        let reader = BufReader::new(file);
        let data: IndexData =
            serde_json::from_reader(reader).map_err(IndexError::SerializationError)?;
        Ok(data)
    }

    /// 内部原子写入：先写 `.tmp`，sync_all，再 rename。使用排他锁防并发。
    fn write_file(path: &Path, data: &IndexData) -> Result<(), IndexError> {
        let tmp = path.with_extension("tmp");
        let file = fs::File::create(&tmp)?;
        file.lock_exclusive().map_err(IndexError::StoreError)?;
        serde_json::to_writer_pretty(&file, data).map_err(IndexError::SerializationError)?;
        file.sync_all().map_err(IndexError::StoreError)?;
        file.unlock().map_err(IndexError::StoreError)?;
        drop(file);
        fs::rename(&tmp, path)?;
        Ok(())
    }
}

impl IndexStore for JsonStore {
    fn read(&self) -> Result<IndexData, IndexError> {
        Self::read_file(&self.path)
    }

    fn write(&self, data: &IndexData) -> Result<(), IndexError> {
        Self::write_file(&self.path, data)
    }

    fn add_project(&mut self, info: ProjectInfo) -> Result<(), IndexError> {
        if self.cache.projects.contains_key(&info.name) {
            return Err(IndexError::AlreadyTracked(info.name));
        }
        self.cache.projects.insert(info.name.clone(), info);
        Self::write_file(&self.path, &self.cache)?;
        Ok(())
    }

    fn remove_project(&mut self, name: &str) -> Result<(), IndexError> {
        if !self.cache.projects.contains_key(name) {
            return Err(IndexError::ProjectNotFound(name.to_string()));
        }
        self.cache.projects.remove(name);
        Self::write_file(&self.path, &self.cache)?;
        Ok(())
    }

    fn get_project(&self, name: &str) -> Option<ProjectInfo> {
        self.cache.projects.get(name).cloned()
    }

    fn list_projects(&self) -> Vec<ProjectInfo> {
        self.cache.projects.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn helper_project(name: &str) -> ProjectInfo {
        ProjectInfo {
            name: name.to_string(),
            path: format!("/tmp/{}", name),
            python_version: Some("3.12".into()),
            toolchain: Some("uv".into()),
            git_remote: None,
            git_branch: None,
            git_status: None,
            deps_count: Some(5),
            created_at: None,
            last_modified: None,
            tags: vec![],
            description: None,
        }
    }

    #[test]
    fn add_and_get_project() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("projects.json");
        let mut store = JsonStore::open(path).unwrap();
        store.add_project(helper_project("a")).unwrap();
        let p = store.get_project("a").unwrap();
        assert_eq!(p.name, "a");
        assert_eq!(p.python_version.unwrap(), "3.12");
    }

    #[test]
    fn add_duplicate_returns_already_tracked() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("projects.json");
        let mut store = JsonStore::open(path).unwrap();
        store.add_project(helper_project("a")).unwrap();
        let err = store.add_project(helper_project("a")).unwrap_err();
        match err {
            IndexError::AlreadyTracked(name) => assert_eq!(name, "a"),
            _ => panic!("expected AlreadyTracked"),
        }
    }

    #[test]
    fn remove_project() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("projects.json");
        let mut store = JsonStore::open(path).unwrap();
        store.add_project(helper_project("a")).unwrap();
        store.remove_project("a").unwrap();
        assert!(store.get_project("a").is_none());
    }

    #[test]
    fn remove_nonexistent_returns_project_not_found() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("projects.json");
        let mut store = JsonStore::open(path).unwrap();
        let err = store.remove_project("noexist").unwrap_err();
        match err {
            IndexError::ProjectNotFound(name) => assert_eq!(name, "noexist"),
            _ => panic!("expected ProjectNotFound"),
        }
    }

    #[test]
    fn list_projects() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("projects.json");
        let mut store = JsonStore::open(path).unwrap();
        store.add_project(helper_project("a")).unwrap();
        store.add_project(helper_project("b")).unwrap();
        let list = store.list_projects();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn round_trip_serialization() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("projects.json");
        let mut store = JsonStore::open(path.clone()).unwrap();
        store.add_project(helper_project("x")).unwrap();

        // 重新打开，验证数据持久化
        let store2 = JsonStore::open(path).unwrap();
        let p = store2.get_project("x").unwrap();
        assert_eq!(p.name, "x");
        assert_eq!(p.toolchain.unwrap(), "uv");
    }

    #[test]
    fn atomic_write_tmp_not_affect_main() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("projects.json");

        // 先写入一个项目
        let mut store = JsonStore::open(path.clone()).unwrap();
        store.add_project(helper_project("a")).unwrap();

        // 写入 .tmp 但不 rename
        let tmp = path.with_extension("tmp");
        let fake_data = IndexData::empty();
        let file = fs::File::create(&tmp).unwrap();
        serde_json::to_writer_pretty(file, &fake_data).unwrap();

        // 重新读取，应该还是 a
        let store2 = JsonStore::open(path).unwrap();
        assert!(store2.get_project("a").is_some());
        assert_eq!(store2.list_projects().len(), 1);
    }

    #[test]
    fn empty_index_on_nonexistent_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");
        let store = JsonStore::open(path).unwrap();
        assert_eq!(store.list_projects().len(), 0);
        assert_eq!(store.cache.version, "1.0");
    }
}
