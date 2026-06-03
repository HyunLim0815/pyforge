//! 目录扫描器。
//!
//! 递归扫描目录并发现 Python 项目。噪声目录会被跳过。

use crate::index::detector::{self, Confidence};
use std::path::{Path, PathBuf};

const SKIP_DIRS: &[&str] = &[".git", "node_modules", "target", "__pycache__", ".venv", "venv"];

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub name: String,
    pub path: PathBuf,
    pub confidence: Confidence,
    pub python_version: Option<String>,
    pub toolchain: Option<String>,
}

/// 扫描目录，`depth` 为最大递归深度；`None` 表示不限制。
pub fn scan(root: &Path, depth: Option<usize>) -> Vec<ScanResult> {
    let mut results = Vec::new();
    scan_inner(root, depth.unwrap_or(usize::MAX), 0, &mut results);
    results.sort_by(|a, b| a.path.cmp(&b.path));
    results
}

fn scan_inner(dir: &Path, max_depth: usize, current_depth: usize, results: &mut Vec<ScanResult>) {
    if current_depth > max_depth || should_skip(dir) {
        return;
    }

    if is_python_project(dir) {
        let detection = detector::detect(dir);
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| dir.display().to_string());
        results.push(ScanResult {
            name,
            path: dir.to_path_buf(),
            confidence: detection.confidence,
            python_version: detection.python_version,
            toolchain: detection.toolchain,
        });
        // 一个目录被识别为项目后，不再深入其内部，避免把子目录误判成多个项目。
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_inner(&path, max_depth, current_depth + 1, results);
        }
    }
}

fn should_skip(dir: &Path) -> bool {
    dir.file_name()
        .map(|name| SKIP_DIRS.contains(&name.to_string_lossy().as_ref()))
        .unwrap_or(false)
}

fn is_python_project(dir: &Path) -> bool {
    dir.join("pyproject.toml").exists()
        || dir.join("setup.py").exists()
        || has_python_file(dir)
}

fn has_python_file(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry
            .path()
            .extension()
            .map(|ext| ext == "py")
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn finds_pyproject_projects() {
        let dir = TempDir::new().unwrap();
        for i in 0..5 {
            let p = dir.path().join(format!("p{i}"));
            fs::create_dir(&p).unwrap();
            fs::write(p.join("pyproject.toml"), "[project]\n").unwrap();
        }
        let found = scan(dir.path(), None);
        assert_eq!(found.len(), 5);
        assert!(found.iter().all(|p| p.confidence == Confidence::High));
    }

    #[test]
    fn detects_mixed_confidence() {
        let dir = TempDir::new().unwrap();
        let high = dir.path().join("high");
        fs::create_dir(&high).unwrap();
        fs::write(high.join("pyproject.toml"), "[project]\n").unwrap();
        let med = dir.path().join("med");
        fs::create_dir(&med).unwrap();
        fs::write(med.join("setup.py"), "").unwrap();
        let low = dir.path().join("low");
        fs::create_dir(&low).unwrap();
        fs::write(low.join("main.py"), "").unwrap();

        let found = scan(dir.path(), None);
        assert_eq!(found.len(), 3);
        assert!(found.iter().any(|p| p.confidence == Confidence::High));
        assert!(found.iter().any(|p| p.confidence == Confidence::Medium));
        assert!(found.iter().any(|p| p.confidence == Confidence::Low));
    }

    #[test]
    fn honors_depth_limit() {
        let dir = TempDir::new().unwrap();
        let deep = dir.path().join("a").join("b").join("c");
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("pyproject.toml"), "[project]\n").unwrap();
        assert_eq!(scan(dir.path(), Some(2)).len(), 0);
        assert_eq!(scan(dir.path(), Some(3)).len(), 1);
    }

    #[test]
    fn skips_noise_dirs() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("target").join("p");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("pyproject.toml"), "[project]\n").unwrap();
        assert_eq!(scan(dir.path(), None).len(), 0);
    }
}
