# PyForge

**Python 项目管家，uv 的强力补充。**

PyForge 是一个 Rust 编写的 CLI 工具，用于管理本地 Python 项目的生命周期——从创建、注册、扫描到依赖分析和 Web 看板，一站式解决多项目管理的痛点。

## 特性

- **项目索引**：注册、扫描、搜索本地 Python 项目，JSON 原子写入 + 文件锁保证数据安全
- **脚手架**：封装 `uv init`，支持内置模板（fastapi/cli/lib）和自定义模板
- **依赖分析**：批量检查过期依赖，跨项目聚合依赖使用情况
- **Web 看板**：本地 Dashboard，项目统计、搜索过滤、依赖分析、一键打开
- **AI Agent 集成**：生成 `SKILL.md` 技能文件，输出 Agent 环境信息
- **i18n**：中英双语内置 + 社区语言包热加载
- **全 JSON 输出**：`--json` 模式可供脚本和 AI Agent 解析

## 安装

### pip 安装（推荐）

```bash
pip install pyproject-manager
```

### 从源码编译

```bash
git clone https://github.com/HyunLim0815/pyforge.git
cd pyforge
cargo install --path .
```

### 前置依赖

- Rust 1.75+
- [uv](https://github.com/astral-sh/uv)（`new` 命令需要）
- [git2](https://docs.rs/git2)（Git 元数据自动采集）

## 快速开始

```bash
# 注册当前目录下的项目
pyforge track ./my-project

# 扫描并批量注册 ~/projects 下所有 Python 项目
pyforge scan ~/projects

# 列出所有已跟踪项目
pyforge list

# 创建新项目（自动 uv init + 注册）
pyforge new my-api --template fastapi

# 启动 Web 看板
pyforge web
```

## 命令一览

| 命令 | 说明 |
|------|------|
| `track <PATH>` | 注册项目到本地索引 |
| `untrack <NAME>` | 从索引中移除项目 |
| `list` | 列出所有已跟踪项目 |
| `scan <DIR>` | 扫描目录并批量注册 Python 项目 |
| `info <NAME>` | 查看项目详细信息 |
| `new <NAME>` | 创建新项目（封装 uv init + 自动注册） |
| `mkpkg <PKG>...` | 在当前项目中批量创建子包 |
| `goto <NAME>` | 输出项目路径（配合 shell 跳转） |
| `status` | 显示所有项目状态概览 |
| `outdated [NAME]` | 检查项目依赖更新状态 |
| `web` | 启动 Web 管理看板（默认端口 7742） |
| `init-agent` | 安装 AI Agent 技能文件 |
| `agent-info` | 输出 AI Agent 环境信息 |
| `i18n list` | 列出可用语言 |
| `i18n install <LANG>` | 从社区仓库安装语言包 |
| `completion <SHELL>` | 生成 Shell 补全脚本 |

## 全局选项

| 选项 | 说明 |
|------|------|
| `--json` | 以 JSON 格式输出（stdout 仅输出 JSON，人类可读信息走 stderr） |
| `--lang <LANG>` | 输出语言（`zh` / `en`），优先级：`--lang` > `PYFORGE_LANG` 环境变量 > 默认中文 |

## 使用示例

### 项目注册与管理

```bash
# 注册项目，自定义名称
pyforge track ./backend --name my-api

# 扫描时自动注册新发现的项目
pyforge scan ~/code --auto-track

# 查看项目详情（JSON 输出）
pyforge info my-api --json

# 移除项目（仅移除索引，不删除文件）
pyforge untrack my-api
```

### 项目创建与脚手架

```bash
# 使用内置模板
pyforge new my-api --template fastapi
pyforge new my-cli --template cli
pyforge new my-lib --template lib

# 不注册到索引
pyforge new scratch --no-track

# 批量创建子包
pyforge mkpkg myapp.models myapp.views myapp.utils
```

### 依赖管理

```bash
# 检查单个项目
pyforge outdated my-api

# 检查所有项目
pyforge outdated

# JSON 输出供脚本使用
pyforge outdated --json
```

### Web 看板

```bash
# 启动（默认自动打开浏览器）
pyforge web

# 指定端口，禁止自动打开
pyforge web --port 8080 --no-open
```

Web 看板提供：
- 项目统计仪表盘（`/api/stats`）
- 项目列表与搜索过滤（`/api/projects`）
- 跨项目依赖聚合分析（`/api/dependencies`）
- 浅色/深色主题切换
- 中英文界面切换

### AI Agent 集成

```bash
# 生成 SKILL.md 技能文件
pyforge init-agent

# 输出环境信息（供 Agent 读取）
pyforge agent-info --json
```

### 国际化

```bash
# 列出可用语言
pyforge i18n list

# 安装日语语言包（示例）
pyforge i18n install ja

# 使用英文输出
pyforge list --lang en

# 通过环境变量切换语言
export PYFORGE_LANG=en
pyforge list
```

## 配置

### 索引路径

默认索引文件位于系统数据目录：
- Windows: `%LOCALAPPDATA%/pyforge/projects.json`
- macOS: `~/Library/Application Support/pyforge/projects.json`
- Linux: `~/.local/share/pyforge/projects.json`

可通过 `PYFORGE_INDEX_PATH` 环境变量覆盖。

### 自定义模板

```bash
# 设置自定义模板目录
export PYFORGE_TEMPLATE_DIR=/path/to/templates

# 创建模板目录结构
mkdir -p $PYFORGE_TEMPLATE_DIR/my-template
# 添加模板文件（支持 {{name}} 变量替换）
echo 'print("Hello {{name}}")' > $PYFORGE_TEMPLATE_DIR/my-template/main.py

# 使用自定义模板
pyforge new my-project --template my-template
```

## 开发

```bash
# 编译
cargo build

# 运行测试（145 个测试）
cargo test

# 代码检查
cargo clippy --all-targets --all-features -- -D warnings

# 格式化
cargo fmt
```

### 项目结构

```
src/
├── main.rs              # 入口
├── cli/                 # CLI 命令实现
│   ├── mod.rs           # clap 路由
│   ├── track.rs         # track 命令
│   ├── list.rs          # list 命令
│   ├── scan.rs          # scan 命令
│   ├── new.rs           # new 命令
│   ├── mkpkg.rs         # mkpkg 命令
│   ├── status.rs        # status 命令
│   ├── outdated.rs      # outdated 命令
│   ├── i18n.rs          # i18n 子命令
│   ├── web.rs           # web 命令
│   └── ...
├── index/               # 索引存储层
│   ├── store.rs         # JSON 原子写入 + 文件锁
│   ├── scanner.rs       # 目录扫描
│   ├── detector.rs      # Python 项目检测
│   └── types.rs         # 数据结构
├── i18n/                # 国际化
│   ├── mod.rs           # 消息查找 + 宏
│   ├── loader.rs        # 外部语言包加载
│   ├── zh.rs            # 中文语料
│   └── en.rs            # 英文语料
├── output/              # 输出格式
│   └── json.rs          # JSON 信封格式
├── templates/           # 项目模板
│   └── mod.rs           # 模板渲染引擎
├── uv/                  # uv CLI 桥接
│   ├── bridge.rs        # 子进程调用
│   └── error.rs         # 错误类型
├── web/                 # Web 看板
│   ├── server.rs        # Axum HTTP 服务
│   └── routes/          # API 路由
└── agent/               # AI Agent 集成
    └── skill.rs         # SKILL.md 生成
```

## 许可证

MIT OR Apache-2.0
