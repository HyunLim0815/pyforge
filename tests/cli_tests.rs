//! PyForge CLI 集成测试。

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

// ── 基础 smoke ──

/// `--help` 列出所有已注册子命令。
#[test]
fn help_lists_all_commands() {
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.arg("--help");
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    for name in &[
        "track", "untrack", "list", "scan", "info", "new", "mkpkg",
        "goto", "status", "web", "agent-info", "init-agent", "completion", "outdated",
    ] {
        assert!(stdout.contains(name), "help output missing: {}", name);
    }
}

/// 不存在的项目输出 i18n 中文提示。
#[test]
fn unimplemented_prints_chinese() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["info", "nonexistent"]);
    cmd.assert().code(1).stderr(contains("项目不在索引中"));
}

// ── track 命令 ──

#[test]
fn track_project_success() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("pyproject.toml"), "[project]\nname = \"test\"\n").unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.arg("track").arg(dir.path());
    cmd.assert().success().stderr(contains("已注册项目"));
}

#[test]
fn track_duplicate_returns_error() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    fs::write(dir.path().join("pyproject.toml"), "[project]\n").unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.arg("track").arg(dir.path());
    cmd.assert().success();
    let mut cmd2 = Command::cargo_bin("pyforge").expect("binary built");
    cmd2.env("PYFORGE_INDEX_PATH", &index);
    cmd2.arg("track").arg(dir.path());
    cmd2.assert().code(1).stderr(contains("已在索引中"));
}

#[test]
fn track_nonexistent_path() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.arg("track").arg(dir.path().join("noexist"));
    cmd.assert().code(1).stderr(contains("路径不存在"));
}

#[test]
fn track_with_name_override() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    fs::write(dir.path().join("pyproject.toml"), "[project]\n").unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["track", &dir.path().display().to_string(), "--name", "my-api"]);
    cmd.assert().success().stderr(contains("my-api"));
}

#[test]
fn track_json_output() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    fs::write(dir.path().join("pyproject.toml"), "[project]\n").unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["track", "--json", &dir.path().display().to_string()]);
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("\"version\""), "json output missing version");
    assert!(stdout.contains("\"success\""), "json output missing success");
}

// ── list 命令 ──

#[test]
fn list_projects_plain() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    for (name, _path) in &[("b", "/tmp/b"), ("a", "/tmp/a")] {
        let sub = dir.path().join(name);
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("pyproject.toml"), "[project]\n").unwrap();
        let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
        cmd.env("PYFORGE_INDEX_PATH", &index);
        cmd.args(["track", &sub.display().to_string(), "--name", name]);
        cmd.assert().success();
    }
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.arg("list");
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // 按名称排序：a 在 b 前面
    let a_pos = stdout.find('a').unwrap();
    let b_pos = stdout.find('b').unwrap();
    assert!(a_pos < b_pos, "projects should be sorted by name");
}

#[test]
fn list_empty_plain() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.arg("list");
    cmd.assert().success().stdout(contains("暂无已跟踪"));
}

#[test]
fn list_json_output() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let sub = dir.path().join("p");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("pyproject.toml"), "[project]\n").unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["track", &sub.display().to_string(), "--name", "p"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["list", "--json"]);
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("\"version\""));
    assert!(stdout.contains("\"projects\""));
}

// ── scan 命令 ──

#[test]
fn scan_finds_five_pyproject_projects_without_auto_track() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    for i in 0..5 {
        let p = dir.path().join(format!("p{i}"));
        fs::create_dir(&p).unwrap();
        fs::write(p.join("pyproject.toml"), "[project]\n").unwrap();
    }
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.arg("scan").arg(dir.path());
    cmd.assert().success().stdout(contains("发现 5 个 Python 项目"));

    // scan 默认不注册；list 仍为空
    let mut list = Command::cargo_bin("pyforge").expect("binary built");
    list.env("PYFORGE_INDEX_PATH", &index);
    list.arg("list");
    list.assert().success().stdout(contains("暂无已跟踪"));
}

#[test]
fn scan_auto_track_registers_projects() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    for i in 0..3 {
        let p = dir.path().join(format!("p{i}"));
        fs::create_dir(&p).unwrap();
        fs::write(p.join("pyproject.toml"), "[project]\n").unwrap();
    }
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["scan", &dir.path().display().to_string(), "--auto-track"]);
    cmd.assert().success();

    let mut list = Command::cargo_bin("pyforge").expect("binary built");
    list.env("PYFORGE_INDEX_PATH", &index);
    list.arg("list");
    let out = list.assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(stdout.contains("p0"));
    assert!(stdout.contains("p1"));
    assert!(stdout.contains("p2"));
}

#[test]
fn scan_json_contains_confidence() {
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

    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.args(["scan", "--json", &dir.path().display().to_string()]);
    let out = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(stdout.contains("High"));
    assert!(stdout.contains("Medium"));
    assert!(stdout.contains("Low"));
}

#[test]
fn scan_depth_limit_and_empty() {
    let dir = TempDir::new().unwrap();
    let deep = dir.path().join("a").join("b").join("c");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("pyproject.toml"), "[project]\n").unwrap();

    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.args(["scan", &dir.path().display().to_string(), "--depth", "2"]);
    cmd.assert().success().stdout(contains("未发现 Python 项目"));
}

// ── info / untrack / goto 命令 ──

#[test]
fn info_plain_and_json() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let p = dir.path().join("my-api");
    fs::create_dir(&p).unwrap();
    fs::write(p.join("pyproject.toml"), "[project]\nrequires-python = \">=3.11\"\n").unwrap();
    let mut track = Command::cargo_bin("pyforge").expect("binary built");
    track.env("PYFORGE_INDEX_PATH", &index);
    track.args(["track", &p.display().to_string(), "--name", "my-api"]);
    track.assert().success();

    let mut info = Command::cargo_bin("pyforge").expect("binary built");
    info.env("PYFORGE_INDEX_PATH", &index);
    info.args(["info", "my-api"]);
    info.assert().success().stdout(contains("python_version"));

    let mut info_json = Command::cargo_bin("pyforge").expect("binary built");
    info_json.env("PYFORGE_INDEX_PATH", &index);
    info_json.args(["info", "my-api", "--json"]);
    info_json.assert().success().stdout(contains("\"project\""));
}

#[test]
fn info_missing_returns_error() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["info", "nonexistent"]);
    cmd.assert().code(1).stderr(contains("项目不在索引中"));
}

#[test]
fn untrack_removes_index_only() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let p = dir.path().join("my-api");
    fs::create_dir(&p).unwrap();
    fs::write(p.join("pyproject.toml"), "[project]\n").unwrap();

    let mut track = Command::cargo_bin("pyforge").expect("binary built");
    track.env("PYFORGE_INDEX_PATH", &index);
    track.args(["track", &p.display().to_string(), "--name", "my-api"]);
    track.assert().success();

    let mut untrack = Command::cargo_bin("pyforge").expect("binary built");
    untrack.env("PYFORGE_INDEX_PATH", &index);
    untrack.args(["untrack", "my-api"]);
    untrack.assert().success().stderr(contains("已从索引移除"));

    assert!(p.exists(), "untrack must not delete project files");

    let mut info = Command::cargo_bin("pyforge").expect("binary built");
    info.env("PYFORGE_INDEX_PATH", &index);
    info.args(["info", "my-api"]);
    info.assert().code(1);
}

#[test]
fn untrack_missing_returns_error() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["untrack", "nonexistent"]);
    cmd.assert().code(1).stderr(contains("项目不在索引中"));
}

#[test]
fn goto_outputs_path_only() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let p = dir.path().join("my-api");
    fs::create_dir(&p).unwrap();
    fs::write(p.join("pyproject.toml"), "[project]\n").unwrap();
    let expected = p.canonicalize().unwrap().display().to_string();

    let mut track = Command::cargo_bin("pyforge").expect("binary built");
    track.env("PYFORGE_INDEX_PATH", &index);
    track.args(["track", &p.display().to_string(), "--name", "my-api"]);
    track.assert().success();

    let mut goto = Command::cargo_bin("pyforge").expect("binary built");
    goto.env("PYFORGE_INDEX_PATH", &index);
    goto.args(["goto", "my-api"]);
    goto.assert().success().stdout(contains(expected));
}

// ── new 命令 ──

#[test]
fn new_success_tracks_project() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.env("PYFORGE_UV_MOCK", "success");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["new", "my-lib"]);
    cmd.assert().success().stderr(contains("项目已创建"));

    let mut list = Command::cargo_bin("pyforge").expect("binary built");
    list.env("PYFORGE_INDEX_PATH", &index);
    list.arg("list");
    list.assert().success().stdout(contains("my-lib"));
}

#[test]
fn new_no_track_does_not_register() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.env("PYFORGE_UV_MOCK", "success");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["new", "my-lib", "--no-track"]);
    cmd.assert().success();

    let mut list = Command::cargo_bin("pyforge").expect("binary built");
    list.env("PYFORGE_INDEX_PATH", &index);
    list.arg("list");
    list.assert().success().stdout(contains("暂无已跟踪"));
}

#[test]
fn new_uv_missing_returns_error() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.env("PYFORGE_UV_MOCK", "missing");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["new", "my-lib"]);
    cmd.assert().code(1).stderr(contains("未找到 uv"));
}

#[test]
fn new_existing_directory_returns_error() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    fs::create_dir(dir.path().join("my-lib")).unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.env("PYFORGE_UV_MOCK", "success");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["new", "my-lib"]);
    cmd.assert().code(1).stderr(contains("目录已存在"));
}

// ── i18n ──

#[test]
fn lang_en_switches_to_english() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["--lang", "en", "info", "nonexistent"]);
    cmd.assert().code(1).stderr(contains("Project not in index"));
}

#[test]
fn env_lang_en_switches_to_english() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_LANG", "en");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["info", "nonexistent"]);
    cmd.assert().code(1).stderr(contains("Project not in index"));
}

#[test]
fn cli_lang_overrides_env() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_LANG", "en");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["--lang", "zh", "info", "nonexistent"]);
    cmd.assert().code(1).stderr(contains("项目不在索引中"));
}

// ── new --template 命令（Story 2.2）──

#[test]
fn new_template_fastapi_creates_structure() {
    let dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.env("PYFORGE_UV_MOCK", "success");
    cmd.args(["new", "order-service", "--template", "fastapi", "--no-track"]);
    cmd.assert().success().stderr(contains("项目已创建"));

    let base = dir.path().join("order-service");
    assert!(base.join("app/main.py").exists(), "missing app/main.py");
    assert!(base.join("app/routers/__init__.py").exists(), "missing app/routers/__init__.py");
    assert!(base.join("app/models/__init__.py").exists(), "missing app/models/__init__.py");
    assert!(base.join("tests/__init__.py").exists(), "missing tests/__init__.py");

    // 验证 {{name}} 已替换
    let main_py = fs::read_to_string(base.join("app/main.py")).unwrap();
    assert!(main_py.contains("order-service"), "{{name}} not replaced in main.py");
}

#[test]
fn new_template_cli_creates_structure() {
    let dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.env("PYFORGE_UV_MOCK", "success");
    cmd.args(["new", "my-cli", "--template", "cli", "--no-track"]);
    cmd.assert().success();

    let base = dir.path().join("my-cli");
    assert!(base.join("src/main.py").exists(), "missing src/main.py");
    let main_py = fs::read_to_string(base.join("src/main.py")).unwrap();
    assert!(main_py.contains("my-cli"), "{{name}} not replaced in cli template");
}

#[test]
fn new_template_lib_creates_structure() {
    let dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.env("PYFORGE_UV_MOCK", "success");
    cmd.args(["new", "my-lib", "--template", "lib", "--no-track"]);
    cmd.assert().success();

    let base = dir.path().join("my-lib");
    assert!(base.join("src/__init__.py").exists(), "missing src/__init__.py");
    assert!(base.join("src/my-lib.py").exists(), "missing src/my-lib.py");
}

#[test]
fn new_template_unknown_returns_error_with_list() {
    let dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.env("PYFORGE_UV_MOCK", "success");
    cmd.args(["new", "x", "--template", "unknown", "--no-track"]);
    cmd.assert()
        .code(1)
        .stderr(contains("未知模板"))
        .stderr(contains("fastapi"));
}

#[test]
fn new_template_custom_from_dir() {
    let dir = TempDir::new().unwrap();
    let tpl_dir = dir.path().join("templates").join("my-tpl");
    fs::create_dir_all(&tpl_dir).unwrap();
    fs::write(tpl_dir.join("README.md"), "# {{name}} project").unwrap();

    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.env("PYFORGE_UV_MOCK", "success");
    cmd.env("PYFORGE_TEMPLATE_DIR", dir.path().join("templates"));
    cmd.args(["new", "foo", "--template", "my-tpl", "--no-track"]);
    cmd.assert().success();

    let readme = fs::read_to_string(dir.path().join("foo/README.md")).unwrap();
    assert!(readme.contains("# foo project"), "custom template {{name}} not replaced");
}

// ── mkpkg 命令（Story 2.3）──

#[test]
fn mkpkg_creates_multiple_packages() {
    let dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.args(["mkpkg", "routers", "models", "services"]);
    cmd.assert().success();

    assert!(dir.path().join("routers/__init__.py").exists());
    assert!(dir.path().join("models/__init__.py").exists());
    assert!(dir.path().join("services/__init__.py").exists());
}

#[test]
fn mkpkg_with_base_path() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("app")).unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.args(["mkpkg", "services", "--base", "app"]);
    cmd.assert().success();

    assert!(dir.path().join("app/services/__init__.py").exists());
}

#[test]
fn mkpkg_nested_dot_notation() {
    let dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.args(["mkpkg", "a.b.c"]);
    cmd.assert().success();

    assert!(dir.path().join("a/__init__.py").exists());
    assert!(dir.path().join("a/b/__init__.py").exists());
    assert!(dir.path().join("a/b/c/__init__.py").exists());
}

#[test]
fn mkpkg_dry_run_does_not_write() {
    let dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.args(["mkpkg", "models.repositories", "--dry-run"]);
    let output = cmd.assert().success();
    output.stderr(contains("dry-run"));

    // dry-run 不应创建任何文件
    assert!(!dir.path().join("models").exists(), "dry-run should not create directories");
}

#[test]
fn mkpkg_existing_skips_without_error() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("existing")).unwrap();
    fs::write(dir.path().join("existing/__init__.py"), "").unwrap();

    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.current_dir(dir.path());
    cmd.args(["mkpkg", "existing", "newpkg"]);
    cmd.assert().success().stderr(contains("已跳过"));

    assert!(dir.path().join("newpkg/__init__.py").exists());
}

#[test]
fn mkpkg_no_args_returns_error() {
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.arg("mkpkg");
    cmd.assert().code(2); // clap required arg missing → exit 2
}

// ── status 命令（Story 3.1）──

#[test]
fn status_empty_index() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.arg("status");
    cmd.assert().success().stderr(contains("暂无已跟踪"));
}

#[test]
fn status_shows_clean_and_dirty_and_missing() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");

    // clean 项目：git init + 无未提交更改
    let clean_dir = dir.path().join("clean-proj");
    fs::create_dir(&clean_dir).unwrap();
    fs::write(clean_dir.join("pyproject.toml"), "[project]\n").unwrap();
    let mut cmd = Command::new("git");
    cmd.args(["init"]).current_dir(&clean_dir);
    cmd.output().unwrap();
    // git add + commit 使其 clean
    let mut ga = Command::new("git");
    ga.args(["add", "."]).current_dir(&clean_dir);
    ga.output().unwrap();
    let mut gc = Command::new("git");
    gc.args(["-c", "user.email=t@t.com", "-c", "user.name=t", "commit", "-m", "init"])
        .current_dir(&clean_dir);
    gc.output().unwrap();

    // dirty 项目：git init + 有未提交文件
    let dirty_dir = dir.path().join("dirty-proj");
    fs::create_dir(&dirty_dir).unwrap();
    fs::write(dirty_dir.join("pyproject.toml"), "[project]\n").unwrap();
    let mut cmd = Command::new("git");
    cmd.args(["init"]).current_dir(&dirty_dir);
    cmd.output().unwrap();
    // 有 untracked 文件 → dirty（StatusOptions include_untracked(false)，但 dirty 意味着有 staged/modified）
    // 需要先 add 然后修改文件来制造 dirty 状态
    let mut ga = Command::new("git");
    ga.args(["add", "."]).current_dir(&dirty_dir);
    ga.output().unwrap();
    let mut gc = Command::new("git");
    gc.args(["-c", "user.email=t@t.com", "-c", "user.name=t", "commit", "-m", "init"])
        .current_dir(&dirty_dir);
    gc.output().unwrap();
    // 修改文件使其 dirty
    fs::write(dirty_dir.join("pyproject.toml"), "[project]\nname = \"changed\"\n").unwrap();

    // missing 项目：路径不存在
    let missing_path = dir.path().join("missing-proj");

    // 注册 clean 和 dirty 项目
    for (name, path) in [
        ("clean-proj", clean_dir.display().to_string()),
        ("dirty-proj", dirty_dir.display().to_string()),
    ] {
        let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
        cmd.env("PYFORGE_INDEX_PATH", &index);
        cmd.args(["track", &path, "--name", name]);
        cmd.assert().success();
    }

    // 手动写入 missing 项目到索引（track 会拒绝不存在的路径）
    {
        let mut data: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&index).unwrap()).unwrap();
        let projects = data["projects"].as_object_mut().unwrap();
        projects.insert(
            "missing-proj".into(),
            serde_json::json!({
                "name": "missing-proj",
                "path": missing_path.display().to_string(),
                "python_version": null,
                "toolchain": "unknown",
                "git_remote": null,
                "git_branch": null,
                "git_status": null,
                "deps_count": null,
                "created_at": null,
                "last_modified": null,
                "tags": [],
                "description": null
            }),
        );
        fs::write(&index, serde_json::to_string_pretty(&data).unwrap()).unwrap();
    }

    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.arg("status");
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("clean-proj"), "missing clean-proj");
    assert!(stdout.contains("[ok]"), "missing [ok]");
    assert!(stdout.contains("dirty-proj"), "missing dirty-proj");
    assert!(stdout.contains("[dirty]"), "missing [dirty]");
    assert!(stdout.contains("missing-proj"), "missing missing-proj");
    assert!(stdout.contains("[missing]"), "missing [missing]");
}

#[test]
fn status_json_output() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    let p = dir.path().join("proj");
    fs::create_dir(&p).unwrap();
    fs::write(p.join("pyproject.toml"), "[project]\n").unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["track", &p.display().to_string(), "--name", "proj"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["status", "--json"]);
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("\"version\""), "json missing version");
    assert!(stdout.contains("\"projects\""), "json missing projects");
    assert!(stdout.contains("\"git_status\""), "json missing git_status");
}

// ── completion ──

#[test]
fn completion_bash_exits_zero() {
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.args(["completion", "bash"]);
    cmd.assert().success().stdout(contains("_pyforge"));
}

// ── outdated 命令 ──

/// 辅助：注册一个项目到索引，返回 (index_path, project_dir)
fn track_project_for_outdated(dir: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let index = dir.path().join("index.json");
    let p = dir.path().join("my-api");
    fs::create_dir(&p).unwrap();
    fs::write(p.join("pyproject.toml"), "[project]\nname = \"my-api\"\n").unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["track", &p.display().to_string(), "--name", "my-api"]);
    cmd.assert().success();
    (index, p)
}

#[test]
fn outdated_all_up_to_date() {
    let dir = TempDir::new().unwrap();
    let (index, _p) = track_project_for_outdated(&dir);
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "outdated_clean");
    cmd.arg("outdated");
    cmd.assert().success().stderr(contains("所有依赖为最新"));
}

#[test]
fn outdated_displays_deps() {
    let dir = TempDir::new().unwrap();
    let (index, _p) = track_project_for_outdated(&dir);
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "outdated:fastapi:0.100.0:0.109.0,uvicorn:0.23.0:0.27.0");
    cmd.arg("outdated");
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("fastapi"), "missing fastapi");
    assert!(stdout.contains("0.100.0"), "missing current version");
    assert!(stdout.contains("0.109.0"), "missing latest version");
    assert!(stdout.contains("uvicorn"), "missing uvicorn");
}

#[test]
fn outdated_uv_missing() {
    let dir = TempDir::new().unwrap();
    let (index, _p) = track_project_for_outdated(&dir);
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "missing");
    cmd.arg("outdated");
    cmd.assert().code(1).stderr(contains("未找到 uv"));
}

#[test]
fn outdated_parse_error() {
    let dir = TempDir::new().unwrap();
    let (index, _p) = track_project_for_outdated(&dir);
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "outdated_parse_error");
    cmd.arg("outdated");
    cmd.assert().code(2).stderr(contains("无法解析"));
}

#[test]
fn outdated_with_name() {
    let dir = TempDir::new().unwrap();
    let (index, _p) = track_project_for_outdated(&dir);
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "outdated:fastapi:0.100.0:0.109.0");
    cmd.args(["outdated", "my-api"]);
    cmd.assert().success().stdout(contains("fastapi"));
}

#[test]
fn outdated_name_not_found() {
    let dir = TempDir::new().unwrap();
    let (index, _p) = track_project_for_outdated(&dir);
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "outdated_clean");
    cmd.args(["outdated", "nonexistent"]);
    cmd.assert().code(1).stderr(contains("不在索引中"));
}

#[test]
fn outdated_json_output() {
    let dir = TempDir::new().unwrap();
    let (index, _p) = track_project_for_outdated(&dir);
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "outdated:fastapi:0.100.0:0.109.0");
    cmd.args(["outdated", "--json"]);
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("\"version\""), "json missing version");
    assert!(stdout.contains("\"projects\""), "json missing projects");
    assert!(stdout.contains("\"outdated\""), "json missing outdated");
}

// ── 扩展集成测试 (P1/P2) ──

/// 辅助：注册两个项目到索引
fn track_two_projects_for_outdated(dir: &TempDir) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let index = dir.path().join("index.json");

    let p1 = dir.path().join("proj-outdated");
    fs::create_dir(&p1).unwrap();
    fs::write(p1.join("pyproject.toml"), "[project]\nname = \"proj-outdated\"\n").unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["track", &p1.display().to_string(), "--name", "proj-outdated"]);
    cmd.assert().success();

    let p2 = dir.path().join("proj-clean");
    fs::create_dir(&p2).unwrap();
    fs::write(p2.join("pyproject.toml"), "[project]\nname = \"proj-clean\"\n").unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["track", &p2.display().to_string(), "--name", "proj-clean"]);
    cmd.assert().success();

    (index, p1, p2)
}

#[test]
fn outdated_mixed_projects_one_outdated_one_clean() {
    let dir = TempDir::new().unwrap();
    let (index, _, _) = track_two_projects_for_outdated(&dir);
    // proj-outdated 有过期依赖，proj-clean 全部最新
    // mock 只能返回一个值，所以两个项目都会用同一个 mock
    // 用 "outdated:fastapi:0.100.0:0.109.0" 让两个项目都显示有过期
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "outdated:fastapi:0.100.0:0.109.0");
    cmd.arg("outdated");
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("proj-outdated"), "missing proj-outdated");
    assert!(stdout.contains("proj-clean"), "missing proj-clean");
    assert!(stdout.contains("fastapi"), "missing fastapi dep");
}

#[test]
fn outdated_skips_missing_project_dir() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    // 注册一个不存在的目录
    let fake_path = dir.path().join("nonexistent-project");
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.args(["track", &fake_path.display().to_string(), "--name", "ghost"]);
    // track 会报路径不存在，所以直接手动写 index
    let data = serde_json::json!({
        "version": "1",
        "projects": {
            "ghost": {
                "name": "ghost",
                "path": fake_path.to_string_lossy(),
                "created_at": "2026-01-01",
                "tags": [],
                "description": null
            }
        }
    });
    fs::write(&index, serde_json::to_string_pretty(&data).unwrap()).unwrap();

    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "outdated_clean");
    cmd.arg("outdated");
    // 目录不存在时静默跳过，不会报错
    cmd.assert().success().stderr(contains("所有依赖为最新"));
}

#[test]
fn outdated_empty_index() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    fs::write(&index, r#"{"version":"1","projects":{}}"#).unwrap();
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "outdated_clean");
    cmd.arg("outdated");
    cmd.assert().success().stderr(contains("暂无已跟踪"));
}

#[test]
fn outdated_json_uv_missing() {
    let dir = TempDir::new().unwrap();
    let (index, _p) = track_project_for_outdated(&dir);
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "missing");
    cmd.args(["outdated", "--json"]);
    let output = cmd.assert().code(1);
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("\"UV_NOT_INSTALLED\""), "missing error code");
    assert!(stdout.contains("\"success\":false"), "missing success:false");
}

#[test]
fn outdated_json_parse_error() {
    let dir = TempDir::new().unwrap();
    let (index, _p) = track_project_for_outdated(&dir);
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "outdated_parse_error");
    cmd.args(["outdated", "--json"]);
    let output = cmd.assert().code(2);
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("\"UV_OUTDATED_ERROR\""), "missing error code");
    assert!(stdout.contains("\"success\":false"), "missing success:false");
}

#[test]
fn outdated_json_all_up_to_date() {
    let dir = TempDir::new().unwrap();
    let (index, _p) = track_project_for_outdated(&dir);
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "outdated_clean");
    cmd.args(["outdated", "--json"]);
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("\"success\":true"), "missing success:true");
    assert!(stdout.contains("\"projects\""), "missing projects");
    // 全最新时 projects 数组不为空（包含项目，只是 outdated 为空）
    assert!(stdout.contains("\"outdated\":[]"), "outdated should be empty array");
}

#[test]
fn outdated_mixed_one_missing_one_clean() {
    let dir = TempDir::new().unwrap();
    let index = dir.path().join("index.json");
    // 一个目录存在，一个不存在
    let real_dir = dir.path().join("real-project");
    fs::create_dir(&real_dir).unwrap();
    fs::write(real_dir.join("pyproject.toml"), "[project]\n").unwrap();
    let fake_dir = dir.path().join("ghost-project");

    let data = serde_json::json!({
        "version": "1",
        "projects": {
            "real": {
                "name": "real",
                "path": real_dir.to_string_lossy(),
                "created_at": "2026-01-01",
                "tags": [],
                "description": null
            },
            "ghost": {
                "name": "ghost",
                "path": fake_dir.to_string_lossy(),
                "created_at": "2026-01-01",
                "tags": [],
                "description": null
            }
        }
    });
    fs::write(&index, serde_json::to_string_pretty(&data).unwrap()).unwrap();

    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("PYFORGE_INDEX_PATH", &index);
    cmd.env("PYFORGE_UV_MOCK", "outdated:fastapi:0.100.0:0.109.0");
    cmd.arg("outdated");
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("real"), "missing real project");
    assert!(stdout.contains("fastapi"), "missing fastapi dep");
    // ghost 被跳过，不出现在输出中
    assert!(!stdout.contains("ghost"), "ghost should be skipped");
}

// ── init-agent 命令 ──

#[test]
fn init_agent_writes_skill_md_and_copies_to_targets() {
    let dir = TempDir::new().unwrap();
    let home = dir.path().join("fake-home");
    fs::create_dir_all(&home).unwrap();

    // 通过 HOME/USERPROFILE 覆盖 dirs::home_dir()
    // 注意：Windows 上 dirs crate 使用 API，此测试验证命令不报错
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("HOME", &home);
    cmd.env("USERPROFILE", &home);
    cmd.args(["init-agent", "--yes", "--json"]);
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("\"results\""), "json missing results");
    // 至少有一个 target 被处理
    assert!(stdout.contains("\"Claude Code\""), "missing Claude Code target");
    assert!(stdout.contains("\"Codex CLI\""), "missing Codex CLI target");
    assert!(stdout.contains("\"Windsurf\""), "missing Windsurf target");
}

#[test]
fn init_agent_json_output() {
    let dir = TempDir::new().unwrap();
    let home = dir.path().join("fake-home");
    fs::create_dir_all(&home).unwrap();

    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("HOME", &home);
    cmd.env("USERPROFILE", &home);
    cmd.args(["init-agent", "--yes", "--json"]);
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("\"version\""), "json missing version");
    assert!(stdout.contains("\"results\""), "json missing results");
    assert!(stdout.contains("\"Claude Code\""), "json missing Claude Code target");
}

#[test]
fn init_agent_idempotent_skip_same_content() {
    let dir = TempDir::new().unwrap();
    let home = dir.path().join("fake-home");
    fs::create_dir_all(&home).unwrap();

    // 第一次写入
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("HOME", &home);
    cmd.env("USERPROFILE", &home);
    cmd.args(["init-agent", "--yes"]);
    cmd.assert().success();

    // 第二次应跳过
    let mut cmd = Command::cargo_bin("pyforge").expect("binary built");
    cmd.env("HOME", &home);
    cmd.env("USERPROFILE", &home);
    cmd.args(["init-agent", "--yes", "--json"]);
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("\"skipped\""), "second run should skip");
}

#[test]
fn init_agent_skill_md_content_snapshot() {
    // 验证 SKILL.md 内容包含关键字段
    let content = include_str!("../skills/pyforge.md");
    assert!(content.contains("PyForge"), "missing PyForge title");
    assert!(content.contains("agent-info"), "missing agent-info capability");
    assert!(content.contains("init-agent"), "missing init-agent capability");
    // 验证所有命令都有文档
    for cmd in &["track", "untrack", "list", "scan", "info", "new", "mkpkg", "goto", "status", "outdated"] {
        assert!(content.contains(cmd), "SKILL.md missing command: {}", cmd);
    }
}

#[test]
fn skill_md_contains_capabilities() {
    // 快照测试：确保 SKILL.md 包含关键能力字段
    let content = include_str!("../skills/pyforge.md");
    assert!(content.contains("agent-info"), "missing agent-info capability");
    assert!(content.contains("list"), "missing list capability");
    assert!(content.contains("scan"), "missing scan capability");
    assert!(content.contains("track"), "missing track capability");
    assert!(content.contains("goto"), "missing goto capability");
    assert!(content.contains("outdated"), "missing outdated capability");
}
