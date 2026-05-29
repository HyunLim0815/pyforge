# PyForge 产品需求文档（PRD）

> **文档版本**：v2.4
> **创建日期**：2026-05-29
> **更新日期**：2026-05-29
> **状态**：final
> **产品名称**：PyForge
> **一句话定位**：Python 项目管家——uv 的强力补充，本地项目索引、智能脚手架、批量子包创建、Web 管理看板、AI Agent 集成，用 Rust 打造极速体验。

## 目录

1. [产品背景与定位](#1-产品背景与定位)
2. [目标用户](#2-目标用户)
3. [核心功能](#3-核心功能)
4. [CLI 接口设计](#4-cli-接口设计)
5. [SKILL.md：AI Agent 集成规范](#5-skillmdai-agent-集成规范)
6. [多语言支持设计](#6-多语言支持设计)
7. [Web 管理看板设计](#7-web-管理看板设计)
8. [数据模型](#8-数据模型)
9. [与 uv 的关系定义](#9-与-uv-的关系定义)
10. [非功能需求](#10-非功能需求)
11. [典型用户故事](#11-典型用户故事)
12. [发布与版本规划](#12-发布与版本规划)
13. [成功指标](#13-成功指标)
14. [风险与缓解](#14-风险与缓解)
15. [Open Questions](#15-open-questions)
16. [Glossary](#16-glossary)
17. [Assumptions Index](#17-assumptions-index)
18. [附录](#18-附录)

---

## 1. 产品背景与定位

### 1.1 生态现状

| 工具 | 擅长 | 不擅长 |
|------|------|--------|
| **uv** | 依赖管理、虚拟环境、Python 版本管理、包安装 | 本地项目发现、项目模板、批量创建子包、项目索引、可视化看板 |
| **Poetry** | 依赖 + 打包一体化 | 无本地多项目管理能力 |
| **PDM** | 类似 Poetry | 同上 |
| **Hatch** | 环境管理 + 模板 | 模板系统复杂，无本地项目索引 |

**空白地带**：没有工具能让你**全面感知本地所有 Python 项目**——它们在哪、什么状态、依赖是否过期、最近何时修改过，并能快速创建标准化的项目骨架或子包。

### 1.2 PyForge 定位

```
┌──────────────────────────────────────────────────────────┐
│                          PyForge                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐ │
│  │ 项目索引  │  │  脚手架  │  │ 子包创建  │  │ Web 看板   │ │
│  │ (track)  │  │  (new)   │  │ (mkpkg)  │  │  (web)     │ │
│  └──────────┘  └──────────┘  └──────────┘  └────────────┘ │
│  ┌──────────────────────────────────────────────────────┐ │
│  │         AI Agent 集成 (SKILL.md + --json)            │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                            │
│           与 uv 无缝协作（uv 管依赖，PyForge 管项目）        │
└──────────────────────────────────────────────────────────────
```

**核心原则**：
- **不替代 uv**，只做 uv 不做的。
- 依赖管理、虚拟环境、Python 版本全部交给 uv。
- PyForge 聚焦：项目发现 → 项目索引 → 项目骨架 → 子包创建 → 可视化看板 → AI Agent 集成。

---

## 2. 目标用户

| 用户角色 | 典型场景 |
|---------|---------|
| **多项目开发者** | 同时维护 5-20 个 Python 项目，经常忘记它们的位置和状态。 |
| **自由职业者/接单者** | 每个客户有多个项目，需要快速回顾已有工作。 |
| **开源贡献者** | 本地 clone 大量仓库，需识别哪些是 Python 项目。 |
| **团队新人** | 接手新电脑，想快速了解本机已有的 Python 项目。 |
| **技术负责人/Tech Lead** | 一眼看清团队项目健康度，无需逐一打开终端。 |
| **AI Agent 使用者** | 通过 Claude Code、Copilot 等 AI 工具自动管理项目。 |

---

## 3. 核心功能

### 3.1 项目索引与跟踪

| ID | 功能 | 优先级 |
|----|------|--------|
| F1 | `pyforge track <path>` — 手动注册项目到本地索引 | P0 |
| F2 | `pyforge list` — 列出所有已跟踪项目 | P0 |
| F3 | `pyforge scan <directory>` — 自动扫描目录发现 Python 项目并批量注册 | P0 |
| F4 | `pyforge info <name>` — 查看项目详细信息（路径、Python 版本、Git 状态、最后修改时间、依赖数、标签） | P1 |
| F5 | `pyforge untrack <name>` — 从索引移除（不删除文件） | P1 |
| F6 | `pyforge goto <name>` — 输出项目绝对路径（配合 `cd` 跳转），仅输出路径字符串 | P1 |
| F7 | 索引数据存储于 `~/.pyforge/projects.json`，人类可读 | P0 |
| F8 | `pyforge list --json` — 输出 JSON，供程序/AI Agent 消费 | P0 |

### 3.2 项目脚手架

| ID | 功能 | 优先级 |
|----|------|--------|
| F9 | `pyforge new <name>` — 创建新项目（内部调用 `uv init` + 自动注册） | P0 |
| F10 | `pyforge new <name> --template fastapi` — 基于 FastAPI 模板创建 `[ASSUMPTION: FastAPI 模板结构已确定：app/main.py, app/routers/, app/models/, tests/]` | P1 |
| F11 | `pyforge new <name> --template cli` — 基于 CLI 工具模板创建 `[ASSUMPTION: CLI 模板使用 `click` 或 `argparse`，待架构决定]` | P1 |
| F12 | 自定义模板存放于 `~/.pyforge/templates/` | P2 |
| F13 | `pyforge new <name> --no-track` — 创建但不注册 | P2 |

### 3.3 子包批量创建

| ID | 功能 | 优先级 |
|----|------|--------|
| F14 | `pyforge mkpkg <packages...>` — 批量创建子包目录 + `__init__.py` | P0 |
| F15 | `pyforge mkpkg <packages...> --base <path>` — 指定父目录 | P0 |
| F16 | `pyforge mkpkg <packages...> --dry-run` — 预览模式 | P1 |
| F17 | 支持嵌套包路径（`a.b.c` → `a/b/c/`） `[ASSUMPTION: 嵌套包路径采用文件系统目录结构，与 Python import 语义一致]` | P1 |

### 3.4 项目状态洞察

| ID | 功能 | 优先级 |
|----|------|--------|
| F18 | `pyforge status` — 所有项目概览（git 状态、最近修改、依赖过期数、最后修改时间） | P1 |
| F19 | `pyforge outdated` — 检查项目依赖更新（通过 CLI 子进程调用 `uv outdated` 并解析 stdout） | P2 |

### 3.5 Web 管理看板

| ID | 功能 | 优先级 |
|----|------|--------|
| F20 | `pyforge web` — 启动本地 Web 服务器，打开看板 | P2 |
| F21 | 仪表盘：项目总数、版本分布、工具链分布、活跃项目 | P2 |
| F22 | 项目卡片列表，含搜索、过滤 | P2 |
| F23 | 项目详情面板（git 信息、依赖列表） | P2 |
| F24 | 一键操作：打开目录、终端、VS Code | P2 |
| F25 | 依赖复用分析、时间线视图 | P2 |
| F26 | 暗色/亮色主题切换 | P2 |

### 3.6 AI Agent 集成

| ID | 功能 | 优先级 |
|----|------|--------|
| F27 | 提供标准化 `SKILL.md` 供 AI Agent 发现 | P0 |
| F28 | 所有查询命令支持 `--json` 标志，输出结构化数据 | P0 |
| F29 | `pyforge agent-info` — 输出 AI Agent 上下文摘要 | P1 |
| F30 | 兼容 Claude Code / Copilot / Codex 等主流 AI 工具 | P1 |

### 3.7 多语言支持

| ID | 功能 | 优先级 |
|----|------|--------|
| F31 | CLI 支持 `--lang <zh|en>` 参数 `[ASSUMPTION: v1.0 通过硬编码消息支持中英文，不依赖可插拔语言包系统]` | P1 |
| F32 | 环境变量 `PYFORGE_LANG` 设置默认语言 `[ASSUMPTION: 优先级：--lang > PYFORGE_LANG > OS 语言检测]` | P1 |
| F33 | 内置简体中文 (zh) 和英文 (en) 完整翻译（消息硬编码，非 JSON 驱动） `[ASSUMPTION: v1.0 不做可插拔 i18n 框架，仅通过 --lang 切换内置消息集]` | P1 |
| F34 | 语言文件存储于 `~/.pyforge/i18n/`，JSON 格式，支持社区贡献 | P2 |
| F35 | `pyforge i18n list` 列出可用语言 | P2 |
| F36 | `pyforge i18n install <lang>` 从社区仓库安装语言包 | P2 |
| F37 | Web 看板支持中英文切换（浏览器语言自动检测） | P2 |

---

### 3.8 P0 功能验收标准

#### 项目索引（F1-F3, F7-F8）

| FR | 验收标准 |
|----|----------|
| **F1** track | `pyforge track /tmp/test-project` → 输出 `✅ 已注册项目: test-project`；`projects.json` 中出现该条目 |
| **F2** list | `pyforge list` → 列出所有已跟踪项目，每行显示名称和路径；`pyforge list --json` → 输出 JSON 数组 |
| **F3** scan | `pyforge scan ~/code` → 输出发现的项目数和自动注册数；`--json` 模式输出结构化结果 |
| **F7** index file | `projects.json` 符合 JSON Schema 校验；手动修改后重新读取不崩溃 |
| **F8** --json output | 所有支持 `--json` 的命令输出合法 JSON；JSON 顶层包含 `version` 和 `data` 字段 |

#### 项目脚手架（F9）

| FR | 验收标准 |
|----|----------|
| **F9** new | `pyforge new my-lib` → 创建目录 `my-lib/`，含 `pyproject.toml`；自动注册到索引；`pyforge list` 能看到 `my-lib` |

#### 子包批量创建（F14-F15）

| FR | 验收标准 |
|----|----------|
| **F14** mkpkg | 在项目目录内执行 `pyforge mkpkg routers models` → 创建 `routers/__init__.py` 和 `models/__init__.py` |
| **F15** --base | `pyforge mkpkg services --base app` → 在 `app/services/__init__.py` 创建 |

#### AI Agent 集成（F27-F28）

| FR | 验收标准 |
|----|----------|
| **F27** SKILL.md | `~/.pyforge/SKILL.md` 存在且格式正确；`pyforge init-agent` 成功复制到目标 AI 工具目录 |
| **F28** --json | 无 `--json` 时人类可读文本；有 `--json` 时仅输出 JSON 到 stdout；非零退出码表示错误 |

---

## 4. CLI 接口设计

### 4.1 命令总览

```
pyforge <COMMAND> [OPTIONS]

Commands:
  track     注册项目到本地索引
  untrack   从索引中移除项目
  list      列出所有已跟踪项目
  scan      扫描目录并批量注册 Python 项目
  info      查看项目详细信息
  new       创建新项目（封装 uv init + 自动注册）
  mkpkg     在当前项目中批量创建子包
  goto      输出项目路径（配合 shell 跳转）
  status    显示所有项目状态概览
  web       启动 Web 管理看板
  agent-info 输出 AI Agent 环境信息
  i18n      语言包管理
  help      帮助信息
```

**全局选项**：
- `--json`：以 JSON 格式输出（适用于 `list`, `info`, `status`, `agent-info`, `scan`）
- `--lang <zh|en>`：指定输出语言

**Shell 补全**：
- 内置 `pyforge completion <shell>` 子命令（支持 bash、zsh、fish、powershell）
- 安装引导提示用户添加 `eval "$(pyforge completion zsh)"` 到 shell 配置

### 4.2 详细命令示例

```bash
# 注册项目
pyforge track /path/to/project
pyforge track . --name my-api

# 列出项目
pyforge list
pyforge list --json

# 扫描目录
pyforge scan ~/code --auto-track
pyforge scan . --depth 2 --json

# 项目详情
pyforge info my-api --json

# 创建项目
pyforge new my-service --template fastapi
pyforge new my-lib --template lib --no-track

# 批量创建子包
pyforge mkpkg database routers middlewares -b app
pyforge mkpkg models.repositories models.services --dry-run

# 状态概览
pyforge status --json

# 跳转路径
cd $(pyforge goto my-api)

# Web 看板
pyforge web
```

---

## 5. SKILL.md：AI Agent 集成规范

### 5.1 文件位置

- 全局技能文件：`~/.pyforge/SKILL.md`
- 安装时可自动复制到 AI 工具目录：
  - Claude Code：`~/.claude/skills/pyforge.md`
  - Codex CLI：`~/.codex/skills/pyforge.md`
  - Windsurf：`~/.windsurf/rules/pyforge.md`

### 5.2 SKILL.md 内容

```markdown
# PyForge Skill for AI Agents

## Metadata
- **Name**: PyForge
- **Version**: 1.0.0
- **Binary**: pyforge
- **Requires**: uv (optional)

## Agent Capabilities
1. **Discover local Python projects** – `pyforge list --json` / `pyforge scan`
2. **Register projects** – `pyforge track`
3. **Create new projects** – `pyforge new` (wraps `uv init`)
4. **Batch create subpackages** – `pyforge mkpkg`
5. **Project status & health** – `pyforge status --json`
6. **Navigate projects** – `pyforge goto <name>`
```

### 5.3 `agent-info` 命令输出

```bash
$ pyforge agent-info --json
```

```json
{
  "pyforge_version": "1.0.0",
  "uv_installed": true,
  "uv_version": "0.4.0",
  "total_projects": 12,
  "index_path": "~/.pyforge/projects.json",
  "templates_available": ["fastapi", "cli", "lib"],
  "recent_projects": [...],
  "system": {"os": "macos", "shell": "zsh", "python_default": "3.12.4"}
}
```

---

## 6. 多语言支持设计

### 6.1 语言文件结构

**位置**：`~/.pyforge/i18n/{lang}.json`

**示例**（`zh.json` 片段）：

```json
{
  "meta": {
    "language": "简体中文",
    "contributors": ["PyForge Team"],
    "version": "1.0.0"
  },
  "strings": {
    "cli.track.success": "✅ 已注册项目: {name}",
    "cli.track.already_tracked": "⚠️ 项目已在索引中: {name}",
    "cli.list.empty": "📋 暂无已跟踪的项目",
    "cli.scan.found": "🔍 发现 {count} 个 Python 项目",
    ...
  }
}
```

---

## 7. Web 管理看板设计

### 7.1 技术架构

- **后端**：Rust 内嵌 axum HTTP 服务器（选型理由：axum 异步、生态成熟、中间件支持好，适合未来扩展；tiny_http 过于简单，不支持 WebSocket 和流式响应），仅监听 `127.0.0.1:7742`
- **前端**：纯 HTML/CSS/JS，无 Node.js 依赖，编译时内嵌进二进制
- **数据**：读取 `~/.pyforge/projects.json`，通过 API 提供实时 git 状态等

### 7.2 主要页面

1. **仪表盘**：统计卡片（项目数、版本分布、工具链）、最近活跃项目 `[ASSUMPTION: 版本分布通过读取各项目 `pyproject.toml` 中 `requires-python` 字段获取]`
2. **项目列表**：卡片展示，支持搜索（名称、路径）、按状态/工具链过滤
3. **项目详情**：完整路径、git 信息、依赖列表、操作按钮（打开目录/终端/IDE）
4. **依赖分析**（可选）：展示跨项目的共同依赖 `[ASSUMPTION: 依赖分析读取各项目的 lock 文件或 `pyproject.toml` 的 dependencies 字段]`

### 7.3 API 端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/` | GET | 主页面 |
| `/api/projects` | GET | 项目列表（支持 `?q=` 搜索, `?toolchain=` 过滤） |
| `/api/projects/:name` | GET | 单个项目详情 |
| `/api/stats` | GET | 统计数据 |
| `/api/open` | POST | 触发打开目录/终端/IDE |
| `/api/agent-info` | GET | 同 `pyforge agent-info --json` |

---

## 8. 数据模型

### 8.1 索引文件：`~/.pyforge/projects.json`

```json
{
  "version": "1.0",
  "projects": {
    "fastapi-prod": {
      "name": "fastapi-prod",
      "path": "/Users/xxx/code/my-fastapi-app",
      "python_version": "3.12.4",
      "toolchain": "uv",
      "git_remote": "git@github.com:user/fastapi-app.git",
      "git_branch": "main",
      "git_status": "clean",
      "deps_count": 23,
      "created_at": "2026-01-15T10:30:00Z",
      "last_modified": "2026-05-28T06:32:00Z",
      "tags": ["production", "fastapi"],
      "description": "Production FastAPI order service"
    }
  }
}
```

---

## 9. 与 uv 的关系定义

```
pyforge new → 内部通过 CLI 子进程调用 `uv init <name>`，解析 stdout/stderr；若 uv 未安装则报错并提示安装
pyforge mkpkg → 纯文件操作（与 uv 无关）
pyforge scan → 识别 uv/Poetry/pip 项目类型
pyforge list/info → 展示 uv 管理的 Python 版本
pyforge web → 只读展示，不改动依赖
pyforge agent-info → 报告环境中的 uv 信息
```

**PyForge 明确不做**：
- 不安装或移除 Python 包
- 不创建、删除或修改虚拟环境
- 不修改 `pyproject.toml` 中的依赖配置

---

## 10. 非功能需求

| 类别 | 要求 |
|------|------|
| **性能** | `list` 100 项目 < 0.5s；`scan` 1000 目录 < 3s；Web 首屏 < 1s |
| **安装** | PyPI (`pip install pyforge`)；Homebrew；提供单二进制下载 |
| **包体积** | < 8 MB（含 Web 资源和语言文件） |
| **跨平台** | Linux、macOS (Intel + Apple Silicon)、Windows |
| **依赖** | 用户端零运行时依赖（Rust 静态链接）；uv 为可选外部依赖 |
| **语言** | CLI：自动检测 + `--lang` 手动切换；Web：浏览器语言检测 + 手动切换 |
| **安全** | Web 服务仅绑定 `127.0.0.1:7742`，不对外暴露；无认证机制（本地信任模型） |
| **可扩展性** | 语言文件、模板、SKILL.md 均可由社区贡献，无需更新主程序。扩展上限：支持最多 50 种语言包、100 个模板 |

---

## 11. 典型用户故事

1. **回顾项目**：小明周一忘记项目位置，`pyforge list` 一秒找到。
2. **新机恢复**：小红复制 `projects.json`，项目索引全部还原。
3. **创建 FastAPI 项目**：小李 `pyforge new order-service --template fastapi`，一步到位。
4. **批量创建子包**：在项目内 `pyforge mkpkg routers models services` 立即生成。
5. **扫描磁盘**：小王 `pyforge scan ~/code --auto-track` 自动注册 23 个项目。
6. **Web 看板**：Tech Lead 老张 `pyforge web` 打开看板，全局查看 15 个微服务状态。
7. **AI Agent 协助**：小明对 Claude Code 说"帮我看看哪些项目有未提交更改"，Agent 执行 `pyforge status --json` 并给出摘要。
8. **多语言切换**：中文环境开发者直接看到中文输出，国外用户默认为英文。

---

## 12. 发布与版本规划

| 版本 | 里程碑 | 核心交付 |
|------|--------|----------|
| v0.1.0 | Alpha | `track` / `list` / `scan` / `mkpkg` + `--json` |
| v0.2.0 | Beta | `new` / `info` / `untrack` |
| v0.3.0 | Beta | `status` / `goto` / `agent-info` |
| v0.4.0 | Beta | 内置中英文 CLI、基本模板系统 |
| v1.0.0 | 正式版 | 全平台测试、CI/CD、Homebrew、SKILL.md 发布 |
| v1.1.0 | 增强 | `outdated`、自定义模板、标签 |
| v1.2.0 | Web 看板 | `web` 命令、仪表盘、项目管理 UI |
| v1.3.0 | 协作 | 索引导出/导入、团队共享 |

---

## 13. 成功指标

| 指标 | 目标 | 类型 |
|------|------|------|
| 周活跃用户（WAU）主动执行 `pyforge` 命令次数 | ≥ 10 次/周 | **产品粘性** |
| 新用户 7 天内创建项目数 | ≥ 2 个 | **产品粘性** |
| PyPI 下载量（12 个月） | 5,000+ | 增长 |
| GitHub Stars | 200+ | 增长 |
| `scan` 扫描速度 | 1000 目录 < 3 秒 | 性能 |
| Web 首屏加载 | < 1 秒 | 性能 |
| AI Agent 兼容工具数 | ≥ 5 | 生态 |
| SKILL.md 被引用次数 | 1,000+ | 生态 |
| 语言包社区贡献数 | ≥ 3 种语言 | 生态 |
| 与 uv 兼容性 | 100% 场景不冲突 | 兼容性 |

**Counter-metrics**（负面追踪指标）：
| 指标 | 阈值 | 说明 |
|------|------|------|
| 初次设置耗时（pip install → 首次 list） | > 2 分钟需优化 | 测量入门摩擦 |
| 与 uv 命令混淆度 | 用户调研中 < 15% 认错 | 确保不增加认知负担 |

---

## 14. 风险与缓解

| 风险 | 缓解措施 |
|------|----------|
| Web 服务安全 | 强制绑定 127.0.0.1；使用最小化 HTTP 库 |
| 索引文件损坏 | JSON Schema 校验 + 自动备份 `.bak` |
| AI Agent 规范变化 | SKILL.md 遵循 Claude Code 标准，多渠道分发 |
| Windows 路径问题 | CI 包含 Windows 测试；全部使用 `PathBuf` |
| 与 uv 版本冲突 | 仅通过 CLI 子进程调用 uv，解析 stdout/stderr，不依赖内部 API |
| uv 竞争风险（uv 未来可能增加项目索引功能） | 差异化为 AI Agent 集成 + Web 看板 + 多项目管理；保持与 uv 的协作定位而非对抗 |

---

## 15. Open Questions

| ID | 问题 | 影响范围 | 决策者 | 期望决策时间 |
|----|------|---------|--------|------------|
| OQ1 | 索引文件并发模型：采用 JSON + 文件锁，还是迁移到 SQLite？JSON 可读性好但并发写入有风险 | §8 | BOSS | 架构设计前 |
| OQ2 | Python 项目检测算法优先级：`pyproject.toml` > `setup.py` > `requirements.txt` > `.py` 文件存在？多语言项目如何识别？ | §3.1 F3 | BOSS | v0.1.0 开发前 |
| OQ3 | Web 看板 CLI 启动后是保持进程常驻还是单次查看后退出？ | §3.5 | BOSS | v1.2.0 规划前 |
| OQ4 | 模板系统：内置模板（编译进二进制）vs 首次运行时远程拉取？ | §3.2 | BOSS | v0.4.0 前 |
| OQ5 | PyPI 包名 `pyforge` 是否已被占用？备选名？ | 发布 | BOSS | v1.0.0 前 |

## 16. Glossary

| 术语 | 定义 |
|------|------|
| **项目索引** | PyForge 维护的本地 Python 项目注册表，存储于 `~/.pyforge/projects.json` |
| **跟踪 (Track)** | 将项目路径注册到索引中，记录元数据但不复制文件 |
| **脚手架 (Scaffolding)** | 通过模板自动生成项目骨架目录和文件 |
| **子包 (Subpackage)** | Python package 内的嵌套模块目录（含 `__init__.py`），对应 import 路径中的 `.` 分隔 |
| **扫描 (Scan)** | 遍历目录树自动发现 Python 项目（依据 `pyproject.toml`/`setup.py` 等特征文件） |
| **AI Agent 集成** | 通过 `--json` 输出和标准 SKILL.md 使 Claude Code 等 AI 工具能发现和操作 PyForge |
| **存量项目 (Brownfield)** | 已存在的 Python 项目，通过 `scan` 或 `track` 导入索引 |

## 17. Assumptions Index

| # | 假设 | 影响的 FR | 验证方式 |
|---|------|----------|---------|
| A1 | `[ASSUMPTION: 用户希望在 Web 看板中一键打开 VS Code]` | F24 | 用户调研确认 |
| A2 | `[ASSUMPTION: 社区会贡献 >3 种语言包]` | F34-F36 | v1.0 发布后观察 PR 数量 |
| A3 | `[ASSUMPTION: `uv outdated` 的输出格式在 uv 版本间保持稳定]` | F19 | CI 中使用固定 uv 版本 |
| A4 | `[ASSUMPTION: 用户愿意在安装时按一次确认来安装 AI Agent 技能]` | F27, Appendix B | Beta 测试验证 |
| A5 | `[ASSUMPTION: 索引文件 ~/.pyforge/projects.json 在下个读写周期完成前不会被其他进程修改]` | F1-F8 | 若出现竞态则迁移到 SQLite |

## 18. 附录

### A. 竞品对比矩阵

| 功能 | PyForge | uv | Poetry | Hatch | PDM |
|------|---------|-----|--------|-------|-----|
| 依赖管理 | ❌ (交给uv) | ✅ | ✅ | ✅ | ✅ |
| 虚拟环境 | ❌ (交给uv) | ✅ | ✅ | ✅ | ✅ |
| Python 版本管理 | ❌ (交给uv) | ✅ | ❌ | ❌ | ❌ |
| 本地项目索引 | ✅ | ❌ | ❌ | ❌ | ❌ |
| 自动项目发现 | ✅ | ❌ | ❌ | ❌ | ❌ |
| 批量创建子包 | ✅ | ❌ | ❌ | ❌ | ❌ |
| 项目模板脚手架 | ✅ | ❌ | ❌ | ✅ | ❌ |
| Web 管理看板 | ✅ | ❌ | ❌ | ❌ | ❌ |
| AI Agent 集成 | ✅ | ❌ | ❌ | ❌ | ❌ |
| 结构化 JSON 输出 | ✅ | 部分 | ❌ | ❌ | ❌ |
| 多语言支持 | ✅ | ❌ | ❌ | ❌ | ❌ |

### B. SKILL.md 分发机制

```
pip install pyforge 后提示用户（而非自动执行）：
 → 询问是否安装 AI Agent 技能文件
 → 手动执行：pyforge init-agent
   （检测已安装的 AI 工具 → 复制技能文件到对应目录）
   • Claude Code → ~/.claude/skills/pyforge.md
   • Codex CLI → ~/.codex/skills/pyforge.md
   • Windsurf → ~/.windsurf/rules/pyforge.md
```

---

**文档结束**

此 PRD 涵盖了 PyForge 的全部核心定义，可作为设计、开发、测试和发布的依据。
