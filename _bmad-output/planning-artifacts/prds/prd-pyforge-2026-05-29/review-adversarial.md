# Adversarial Review — PyForge PRD

## Overall verdict

这份 PRD 最大的问题不是它做错了什么，而是它**没有思考的东西太多**。它假设"Python 项目管家"这个定位天然成立，假设用户会记住 `pyforge` 这个新命令，假设 Rust 编译的单二进制在任何环境下都能正常工作。更严重的是——**竞品其实正在填补这段空白**。uv 一年后也可能推出项目索引功能，届时要怎么竞争？

## Findings

### [critical] 没有考虑 uv 的竞争反应 — §1, §9

PRD 假设 uv 永远只做依赖管理，但这没有依据。uv 已经从一个 pip 替代品成长为一个生态系统（支持 `uv run`、`uv build`、`uv publish`）。uv 团队完全可以在一年内增加 `uv project list` 或 `uv init --template` 功能。届时 PyForge 的"空白地带"将不复存在。

*Fix:* 要么明确差异化策略（PyForge 不追求成为 uv 的一部分，而是成为 IDE-agnostic 的项目发现层），要么承认这个风险。

### [high] 索引文件的竞态条件和数据一致性问题 — §8, §7

`~/.pyforge/projects.json` 被 CLI 写入、被 Web 看板读取、可能同时被多个终端会话修改。如果 `pyforge scan` 耗时 3 秒，在此期间另一个终端执行 `pyforge track`，会发生什么？整个文件可能被覆盖或损坏。JSON Schema 校验和 `.bak` 备份（§14）能缓解但不能解决竞态。

*Fix:* 要么使用文件锁（`fs2` crate），要么改用 SQLite（单个连接、事务支持）。对于"人类可读"的需求，JSON + 锁足够，但这是重大架构决策，需要在 PRD 中明确。

### [high] 没有定义"Python 项目"的识别标准 — §3.1 F3

`pyforge scan` 依赖一个隐含分类器：什么算"Python 项目"？包含 `pyproject.toml` 的目录？包含 `setup.py` 的目录？包含任意 `.py` 文件的目录？如果同目录下有 `pyproject.toml` 和 `package.json` 怎么办？如果目录既是 Python 项目又是 Node.js monorepo 的一部分？这些边缘情况没有在 PRD 中处理。

*Fix:* 明确定义"Python 项目的识别规则层级"：`pyproject.toml` > `setup.py/setup.cfg` > `requirements.txt` > 包含 `.py` 文件的目录（低置信度），并说明如何处理多语言项目。

### [high] Web 看板是过度的分布式架构 — §3.5, §7

对于"列表 100 个项目"这个核心场景，启动一个 Web 服务器、打开浏览器、通过 HTTP API 读取 JSON 文件，远比 `pyforge list` 重。这个功能对 Tech Lead 有吸引力，但：
1. **启动时间**：Rust 启动 HTTP 服务器需要时间，抵消了"快速查看"的体验
2. **维持运行**：PRD 没有说明 Web 看板是"启动-查看-关闭"还是"后台常驻"
3. **与 CLI 的重叠**：F21-F25 的大部分功能（搜索、过滤、详情）CLI 版本也能做到

*Fix:* 将 Web 看板降级为 P2，先确保核心 CLI 体验完美后再考虑。

### [medium] i18n 的优先级过高 — §3.7, §6

F31-F37（7 个 FR）分配给 i18n，其中 F33（内置中英文完整翻译）标记为 P1。对于一个 CLI 工具，在 v1.0 之前投入大量精力做翻译系统，而核心功能（F18 status、F19 outdated、F24 一键操作）还只是 P1/P2——这暴露了优先级失衡。一个 CLI 工具的国际化可以通过简单的英语输出 + 文档翻译解决，不需要完整的 i18n 框架。

*Fix:* 将完整的 i18n 框架（F34-F37）降到 v1.1+，v1.0 仅通过命令行 `--lang` + 硬编码消息支持中英文切换，不做可插拔语言包系统。

### [medium] 没有离线/降级策略 — §3, §7

所有核心功能（track、list、scan、web）都是本地操作，看似不需要联网——但":
1. `pyforge scan` 扫描 Git 仓库时可能需要联网获取远程信息？
2. `pyforge new --template fastapi` 的模板从哪里来？内置在二进制中还是从远程下载？
3. 索引文件损坏（§14 已提及但未说明）的恢复流程是什么？

*Fix:* 明确哪些功能需要联网、哪些完全离线。模板的获取策略（内置 vs 远程拉取）需要决策。

### [medium] "用 Rust 打造"是最初的唯一理由但也是约束 — 全文

PRD 在开篇就声明"用 Rust 打造极速体验"，但没有论证**为什么 Rust 是正确答案**。对于文件索引、JSON 读写、CLI 路由这些功能，Go、Python（用 Click + JSON）、甚至 Node.js 都能胜任。选择 Rust 带来了：
1. 团队需要 Rust 技能（如果是个人项目，风险可控但需要承认）
2. 编译时间
3. 内嵌 HTTP 服务器（Web 功能的依赖）
4. 跨平台编译复杂度（尤其是 Apple Silicon + Windows + Linux 的 triple target 需要通过 CI 验证）

*Fix:* 在 §10 或附录中增加技术选型理由文档。即使"因为我想学 Rust"也是一个 honest answer。

### [low] 没有 Shell 补全的设计 — §4

PRD 列出了丰富的 CLI 命令和全局选项，但没有提到 Shell 补全（bash/zsh/fish）。对于开发者工具，Shell 补全几乎是从第一天就应该有的体验。

*Fix:* 在 §4 或 §10 中增加 Shell 补全支持（clap 的 `complete` 子命令），附加说明是否在 v0.1.0 还是 v0.2.0 加入。

### [low] SKILL.md 分发机制假设过多 — §5, 附录 B

附录 B 假设安装后自动检测并写入 `~/.claude/skills/pyforge.md`、`~/.codex/skills/pyforge.md` 等。但：
1. 用户可能使用其他 AI 工具（Cursor、Windsurf 更新频繁）
2. 在 Windows 上路径完全不同
3. 安装过程写入用户 home 目录下的隐藏配置属于侵入性操作，用户可能不期望
4. 权限问题（用户以 sudo 安装但 AI 配置在用户 home 下）

*Fix:* 将自动分发改为"安装时提示"（opt-in），并提供 `pyforge init-agent` 命令手动设置。

### [low] 没有性能退化测试策略 — §10

NFR 指定了 `list 100 项目 < 0.5s` 和 `scan 1000 目录 < 3s`，但没有说明如何在 CI 中保证这些指标不过线。随着项目数增长或文件系统变深，性能退化是渐进式的，不通过 CI 门禁很难发现。

*Fix:* 在 §14 或 CI 章节中增加性能回归测试策略，说明基准测试工具和 fail threshold。
