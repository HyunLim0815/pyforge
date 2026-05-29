---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
inputDocuments:
  - "_bmad-output/planning-artifacts/prds/prd-pyforge-2026-05-29/prd.md"
workflowType: 'architecture'
project_name: 'pyforge'
user_name: 'BOSS'
date: '2026-05-29'
lastStep: 8
status: 'complete'
completedAt: '2026-05-29'
---

# PyForge Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements (37 FRs, 4 categories):**
- Project Indexing (F1-F8): P0 core — track, list, scan, JSON output
- Project Scaffolding (F9-F13): P0 for `new`, P1-P2 for templates
- Subpackage Creation (F14-F17): P0 core — mkpkg with --base and nested paths
- Status Insights (F18-F19): P1-P2 — status overview, outdated check
- Web Dashboard (F20-F26): P2 deferred — axum-based
- AI Agent Integration (F27-F30): P0 for SKILL.md + --json
- i18n (F31-F37): P1 hardcoded messages (v1.0), P2 framework deferred

**Non-Functional Requirements:**
- Performance: `list` 100 projects < 0.5s, `scan` 1000 dirs < 3s, Web FCP < 1s
- Binary size: < 8MB (static-linked Rust)
- Cross-platform: Linux, macOS (Intel + Apple Silicon), Windows
- Zero runtime dependencies (Rust static linking, uv is optional external)
- Security: localhost-only binding, no auth

**Scale & Complexity:**
- Complexity: Low-Medium (single-user local CLI tool)
- Primary domain: CLI tool + optional local web
- Estimated architecture components: 4-5 (index engine, CLI router, uv bridge, web server, i18n)

### Technical Constraints & Dependencies

| Constraint | Source |
|---|---|
| Rust language (static-linked, < 8MB) | PRD §10 |
| uv via CLI subprocess only | PRD §9 |
| axum for web framework | PRD §7.1 |
| Index file at `~/.pyforge/projects.json` | PRD §8 |
| Localhost-only binding (127.0.0.1) | PRD §14 |

### Cross-Cutting Concerns

- Index file consistency (single-user concurrent access)
- Cross-platform path handling (PathBuf + CI matrix)
- CLI output format duality (human text vs --json)
- Error handling & user feedback UX
- i18n message system (hardcoded for v1.0)

### Key Architectural Decisions Made (Party Mode)

| OQ | Decision | Detail |
|----|----------|--------|
| OQ2 | Auto confidence grading for Python detection | ✅ High (pyproject.toml) / Medium (setup.py) / Low (.py only) — user decides auto-track for low confidence |

## Starter Template Evaluation

### Primary Technology Domain

Rust CLI tool (with optional embedded axum web server)

### Core Crate Stack

| Category | Crate | Purpose | Rationale |
|---|---|---|---|
| CLI parsing | `clap` (derive) | Command routing, arg parsing, shell completion | Derive API matches PRD command structure; built-in `complete` subcommand |
| Serialization | `serde` + `serde_json` | `projects.json` read/write | Industry standard; PRD §8 requires human-readable JSON |
| Async runtime | `tokio` | Parallel `scan`, axum web server | Required by axum ecosystem |
| Web framework | `axum` | Web dashboard (P2, arch预留) | ✅ Already decided (PRD §7.1) |
| Config parsing | `toml` + `serde` | `pyproject.toml` parsing | Required for Python project detection |
| Git integration | `git2` | `status` git state detection | Faster and more cross-platform-consistent than CLI subprocess |
| Cross-platform paths | `dirs` | `~/.pyforge/` resolution | Platform-aware path resolution |
| File locking | `fs2` | Index concurrency (v0.1) | Pairs with atomic-rename strategy |
| CLI testing | `assert_cmd` + `predicates` | Integration tests | Standard Rust CLI test toolchain |
| Structured logging | `tracing` | Debuggability | Modern context-aware logging over `log` crate |

### Project Structure

```
pyforge/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli/                  # Command definitions (clap derive)
│   │   ├── mod.rs
│   │   ├── track.rs
│   │   ├── list.rs
│   │   ├── scan.rs
│   │   ├── new.rs
│   │   └── mkpkg.rs
│   ├── index/                # Index engine
│   │   ├── mod.rs
│   │   ├── store.rs          # IndexStore trait + JSON impl
│   │   ├── detector.rs       # Python detection (confidence grading)
│   │   └── scanner.rs        # Directory traversal
│   ├── uv/                   # uv bridge
│   │   ├── mod.rs
│   │   └── bridge.rs         # CLI subprocess invocation
│   ├── web/                  # Web dashboard (P2,预留)
│   │   ├── mod.rs
│   │   └── server.rs
│   └── i18n/                 # i18n (v1.0 hardcoded)
│       ├── mod.rs
│       └── messages.rs
├── tests/
│   ├── cli_tests.rs          # Integration tests
│   └── index_tests.rs
└── templates/                # Embedded templates (include_str!)
    ├── fastapi/
    └── cli/
```

### Dependency Version Strategy

- Workspace-level exact version pinning
- CI enforces `cargo deny` for security advisories
- Automated Dependabot/Renovate for minor updates

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**

| Decision | Choice | Rationale |
|---|---|---|
| OQ1: Index concurrency model | **JSON + atomic rename** | Zero extra deps, satisfies F7 (human-readable), microsecond window acceptable for v0.1 single-user |
| OQ2: Python detection algorithm | **Confidence grading** | High (pyproject.toml) / Medium (setup.py) / Low (.py only); user controls auto-track for low confidence |

**Important Decisions (Shape Architecture):**

| Decision | Choice | Rationale |
|---|---|---|
| IndexStore trait | Abstracted behind trait for future migration | JSON impl for v0.1; SQLite impl possible later without CLI code changes |
| CLI test strategy | `assert_cmd` + `predicates` | Standard Rust CLI test toolchain |
| Async runtime | `tokio` (full features) | Required by axum; `scan` parallelism benefits from async traversal |

**Deferred Decisions (Post-MVP):**

| Decision | Status | Notes |
|---|---|---|
| OQ3: Web dashboard process model | Deferred (P2) | "Spawn → view → Ctrl+C" model, like `python -m http.server` |
| OQ4: Template distribution | Deferred (P2) | Embedded in binary via `include_str!` for v1.0 |
| i18n framework (F34-F37) | Deferred (P2) | v1.0: hardcoded messages with --lang switch |

### Data Architecture

**Index File Format:** JSON (`~/.pyforge/projects.json`)
**Write Strategy:** Write to `.tmp` → `std::fs::rename` (atomic on POSIX, atomic on same-partition Windows)
**Read Strategy:** Direct `serde_json::from_reader` — stale data acceptable for CLI tool semantics
**Schema Version:** `projects.json` top-level `"version"` field for forward compatibility
**Location:** `dirs::data_dir().join("pyforge").join("projects.json")` (PRD says `~/.pyforge/` — resolved via `dirs` crate)

### Infrastructure & Deployment

**CI Platform:** GitHub Actions
**Cross-Platform Matrix:** ubuntu-latest, macos-latest, windows-latest
**Binary Distribution:** GitHub Releases (per-platform artifacts)
**PyPI Distribution:** `maturin build` → native binary wheels for linux/macos/windows
**Homebrew:** Custom tap (`pyforge/tap`) for v1.0; homebrew-core submission post-v1.0
**Versioning:** Semantic versioning via Conventional Commits
**Security Auditing:** `cargo deny` in CI pipeline

### Decision Impact Analysis

**Implementation Sequence:**
1. `src/index/store.rs` + `src/index/detector.rs` — Index engine (core)
2. `src/cli/` — CLI routing via clap derive
3. `src/uv/bridge.rs` — uv subprocess integration
4. `src/index/scanner.rs` — Parallel directory traversal
5. `src/i18n/messages.rs` — Hardcoded i18n
6. `src/web/` — axum server (P2, last)

**Cross-Component Dependencies:**
- All CLI commands depend on `IndexStore` trait
- `scan` depends on `detector.rs` (confidence grading)
- `new` depends on `uv/bridge.rs`
- `status` depends on `git2` integration
- Web depends on `IndexStore` + `uv/bridge`

## Implementation Patterns & Consistency Rules

### Naming Patterns

| Category | Convention | Example |
|---|---|---|
| Types / Structs | `PascalCase` | `IndexStore`, `ProjectInfo` |
| Functions / Methods | `snake_case` | `write_index()`, `scan_directory()` |
| Modules | `snake_case` | `src/cli/track.rs` |
| CLI commands | kebab-case | `pyforge agent-info` |
| JSON fields | `snake_case` | `python_version`, `last_modified` |
| Error types | `PascalCase + Error` | `IndexError`, `ScanError` |
| `#[derive]` on public types | `Debug, Clone` as minimum; add `Serialize, Deserialize` for JSON types | `#[derive(Debug, Clone, Serialize, Deserialize)]` on `ProjectInfo` |

### Structure Patterns

| Rule | Detail |
|---|---|
| Module = subsystem | Each core subsystem is a module dir under `src/` |
| Minimal public API | `pub use` only in `mod.rs`; implementation details are `pub(crate)` |
| Error type per module | Each subsystem defines its own error enum implementing `From` → top-level error |
| Tests follow module | Unit tests in `#[cfg(test)] mod tests` at module bottom; integration tests in `tests/` |
| Integration tests use `tempfile::TempDir` | Never write to real `~/.pyforge/` in tests — always use temp directories |

### JSON Output Format (for --json flag)

```json
// Success
{
  "version": "1.0",
  "success": true,
  "data": {
    // command-specific payload
  }
}

// Error
{
  "version": "1.0",
  "success": false,
  "error": {
    "code": "PROJECT_NOT_FOUND",
    "namespace": "INDEX",
    "message": "项目 'foo' 不在索引中"
  }
}
```

**Error Code Namespaces:**

| Namespace | Error Codes | Source |
|---|---|---|
| `INDEX_*` | `INDEX_PROJECT_NOT_FOUND`, `INDEX_ALREADY_TRACKED`, `INDEX_STORE_ERROR` | F1-F8 |
| `SCAN_*` | `SCAN_DIR_NOT_FOUND`, `SCAN_PERMISSION_DENIED`, `SCAN_NO_PROJECTS` | F3 |
| `UV_*` | `UV_NOT_INSTALLED`, `UV_INIT_FAILED`, `UV_OUTDATED_ERROR` | F9, F19 |
| `CLI_*` | `CLI_INVALID_ARG`, `CLI_COMMAND_FAILED` | §4 |
| `WEB_*` | `WEB_SERVER_ERROR` | F20 (P2) |

### Process Patterns

| Pattern | Rule |
|---|---|
| Exit codes | 0 = success, 1 = user error, 2 = system error |
| stderr | Error messages + progress output (for `scan`) |
| stdout | Command output only (`list`, `status`, `goto`) — never mix with diagnostic output |
| Human output | Always in `{communication_language}` (Chinese for v1.0) via centralized messages |
| --json output | ONLY JSON to stdout; all human messages go to stderr or are suppressed |

### Logging Patterns (tracing)

| Level | Usage |
|---|---|
| `error!` | Unrecoverable errors — always paired with a user-facing error message |
| `warn!` | Recoverable issues (e.g., `pyproject.toml` parse failure, skip file) |
| `info!` | CLI command entry/exit — logged once per command execution |
| `debug!` | Internal state dumps, serialization payloads, subprocess output |
| `trace!` | Loop iterations, per-file scanning progress |

**Span Conventions:** Each CLI command opens a span named `cmd::{command_name}` at `info!` level.

### i18n Message Pattern

All user-visible strings centralized in `src/i18n/messages.rs`:

```rust
// Pattern: msg("section.key", &[arg1, arg2])
pub fn msg(key: &str, args: &[&str]) -> String {
    match (key, LANG.load()) {
        ("track.success", _) => format!("✅ 已注册项目: {}", args[0]),
        ("track.already_tracked", _) => format!("⚠️ 项目已在索引中: {}", args[0]),
        ("list.empty", _) => "📋 暂无已跟踪的项目".to_string(),
        ("scan.found", _) => format!("🔍 发现 {} 个 Python 项目", args[0]),
        _ => key.to_string(),
    }
}
```

**Rationale:** Centralizing messages means future i18n framework upgrades (F34-F37, P2) only need to change one file. CLI commands MUST NOT embed user-facing strings inline.

### Enforcement Guidelines

**All AI Agents MUST:**
- Use `tempfile::TempDir` for all integration tests involving the index file
- Add `#[derive(Debug, Clone)]` to every public struct; add `Serialize, Deserialize` to JSON structs
- Route all user-facing strings through `i18n::msg()` — never inline Chinese or English text
- Use `tracing::info_span!("cmd::{name}")` at the start of each CLI command handler
- Define error codes in the correct namespace prefix before adding new error variants

## Project Structure & Boundaries

### Complete Project Directory Structure

```
pyforge/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── .gitignore
├── .github/workflows/
│   ├── ci.yml                     # Cross-platform matrix tests
│   └── release.yml                # GitHub Release + PyPI publish
├── deny.toml                      # cargo deny config
├── clippy.toml                    # Clippy lint config
├── rust-toolchain.toml            # Rust version pinning
├── src/
│   ├── main.rs                    # CLI entry point
│   ├── cli/                       # Command definitions (clap derive)
│   │   ├── mod.rs
│   │   ├── track.rs               # F1
│   │   ├── list.rs                # F2/F8
│   │   ├── scan.rs                # F3
│   │   ├── info.rs                # F4
│   │   ├── untrack.rs             # F5
│   │   ├── goto.rs                # F6
│   │   ├── new.rs                 # F9-F13
│   │   ├── mkpkg.rs               # F14-F17
│   │   ├── status.rs              # F18
│   │   ├── web.rs                 # F20 (P2 stub)
│   │   ├── agent_info.rs          # F29
│   │   └── completion.rs          # Shell completion
│   ├── index/                     # Index engine
│   │   ├── mod.rs
│   │   ├── store.rs               # IndexStore trait + JSON impl
│   │   ├── detector.rs            # Python detection (confidence grading)
│   │   ├── scanner.rs             # Directory traversal (tokio parallel)
│   │   ├── types.rs               # ProjectInfo, IndexData structs
│   │   └── error.rs               # IndexError enum
│   ├── uv/                        # uv bridge
│   │   ├── mod.rs
│   │   ├── bridge.rs              # CLI subprocess invocation
│   │   └── error.rs               # UvError enum
│   ├── web/                       # Dashboard (P2, reserved)
│   │   ├── mod.rs
│   │   ├── server.rs
│   │   ├── routes.rs
│   │   └── error.rs
│   ├── i18n/                      # i18n (v1.0 hardcoded)
│   │   ├── mod.rs                 # msg() function
│   │   ├── zh.rs                  # Chinese messages
│   │   └── en.rs                  # English messages
│   ├── output/                    # JSON output formatting
│   │   ├── mod.rs
│   │   ├── json.rs                # JsonOutput serialization
│   │   └── error.rs               # Top-level Error enum
│   └── config.rs                  # Config loading (dirs + paths)
├── templates/                     # Embedded templates (include_str!)
│   ├── fastapi/
│   └── cli/
└── tests/
    ├── cli_tests.rs               # CLI integration tests
    ├── index_tests.rs             # Index engine integration tests
    └── common/mod.rs              # Shared test utilities
```

### Requirements → Structure Mapping

| PRD FR Group | Files | Boundary |
|---|---|---|
| F1-F8 Indexing | `src/index/` + `src/cli/{track,list,scan,info,untrack,goto}.rs` | IndexStore trait |
| F9-F13 Scaffolding | `src/cli/new.rs` → `src/uv/bridge.rs` | uv CLI subprocess |
| F14-F17 Subpackages | `src/cli/mkpkg.rs` | Filesystem only |
| F18-F19 Status | `src/cli/status.rs` → `src/index/` + `git2` | Git state (external) |
| F20-F26 Web | `src/web/` (P2 stub) | Axum internal server |
| F27-F30 AI Agent | `src/cli/agent_info.rs`, `src/output/json.rs` | JSON output contract |
| F31-F37 i18n | `src/i18n/` | msg() single entry point |

### Integration Boundaries

| Boundary | Entry | Exit |
|---|---|---|
| CLI ↔ Index engine | `IndexStore` trait | JSON file |
| CLI ↔ uv | `UvBridge` (CLI subprocess) | stdout/stderr parse |
| CLI ↔ User | stdout (data) / stderr (progress+error) | Exit code |
| CLI ↔ AI Agent | `--json` flag → `JsonOutput` | stdout JSON |
| CLI ↔ Web (P2) | `src/web/routes.rs` → `IndexStore` | Axum HTTP |
| Module ↔ Logging | `tracing` macros | `tracing` subscriber |

## Architecture Validation Results

### Coherence Validation ✅

- **Decision Compatibility:** All technology choices (clap + serde + tokio + axum + git2) are compatible — tokio is a transitive dependency of axum, no version conflicts
- **Pattern Consistency:** Rust naming conventions (snake_case, PascalCase) naturally align with clap derive API and serde JSON serialization
- **Structure Alignment:** Module-per-subsystem structure directly maps to architectural boundaries (IndexStore trait, UvBridge, JsonOutput)

### Requirements Coverage Validation ✅

- **P0 FRs (10):** All have direct file-level assignments and acceptance criteria (§3.8 of PRD)
- **P1 FRs (13):** All mapped to specific CLI modules with priority markers
- **P2 FRs (14):** Deferred with clear rationale — Web (stub), i18n framework, templates
- **NFRs:** Performance (< 0.5s/3s/1s) addressed via in-memory cache + tokio parallel; < 8MB via minimal deps; cross-platform via PathBuf + CI matrix

### Implementation Readiness Validation ✅

- **Decisions:** All 4 Open Questions resolved (OQ1: atomic rename, OQ2: confidence grading, OQ3/OQ4: deferred to P2)
- **Patterns:** 7 pattern categories defined (naming, structure, JSON output, process, logging, i18n, enforcement)
- **Structure:** 38+ files in complete tree with FR-to-file mapping
- **Boundaries:** 6 integration boundaries documented with entry/exit points

### Gap Analysis

| Priority | Gap | Status |
|---|---|---|
| Critical | None found | ✅ |
| Important | Homebrew formula specifics in CI release workflow | ⏳ Deferred to CI setup |
| Minor | Exact Rust version in rust-toolchain.toml | ⏳ Set at dev start |

### Architecture Completeness Checklist

- [x] Requirements analysis — scope, constraints, cross-cutting concerns
- [x] Critical decisions documented with rationale
- [x] Technology stack fully specified
- [x] Integration patterns defined
- [x] Naming conventions established
- [x] Structure patterns defined
- [x] Communication patterns specified
- [x] Process patterns documented
- [x] Complete directory structure defined
- [x] Component boundaries established
- [x] Integration points mapped
- [x] Requirements-to-structure mapping complete

### Architecture Readiness Assessment

**Overall Status:** ✅ READY FOR IMPLEMENTATION
**Confidence Level:** High

**Key Strengths:**
- IndexStore trait abstraction enables future SQLite migration without CLI changes
- Unified tempfile-based test strategy prevents test pollution
- Centralized i18n msg() function provides upgrade path to P2 JSON-driven framework
- Error code namespaces prevent AI agent naming conflicts

**First Implementation Priority:**
1. `src/index/types.rs` + `src/index/store.rs` — IndexStore trait + atomic-rename JSON impl
2. `src/cli/track.rs` + `src/cli/list.rs` — First usable commands
