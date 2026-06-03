# PyForge Agent Skill

PyForge is a Python project management CLI tool built in Rust.

## Capabilities

- **Project Tracking**: Register, list, scan, and manage Python projects
- **Scaffolding**: Create new projects with templates (fastapi, cli, lib)
- **Dependency Analysis**: Check outdated deps, cross-project aggregation
- **Web Dashboard**: Local management UI with stats and search
- **i18n**: Multi-language support (zh, en, + community packs)

## Commands

- `track <PATH>` - Register a project
- `untrack <NAME>` - Remove project from index
- `list` - List all tracked projects
- `scan <DIR>` - Scan directory for Python projects
- `info <NAME>` - Show project details
- `new <NAME>` - Create new project (wraps uv init)
- `mkpkg <PKG>...` - Create sub-packages
- `status` - Show project status overview
- `outdated [NAME]` - Check dependency updates
- `web` - Start web dashboard (port 7742)
- `init-agent` - Install agent skill files
- `agent-info` - Output agent environment info

## JSON Output

All commands support `--json` flag for machine-readable output.
Human-readable output goes to stderr, JSON to stdout.

## Environment Variables

- `PYFORGE_LANG` - Language (zh/en)
- `PYFORGE_INDEX_PATH` - Custom index file path
- `PYFORGE_TEMPLATE_DIR` - Custom template directory
- `PYFORGE_UV_MOCK` - Mock uv responses (testing)
