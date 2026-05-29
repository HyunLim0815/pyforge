# Validation Report — PyForge PRD

- **PRD:** `_bmad-output/planning-artifacts/prds/prd-pyforge-2026-05-29/prd.md`
- **Rubric:** `.claude/skills/bmad-prd/assets/prd-validation-checklist.md`
- **Run at:** 2026-05-29T12:00:00Z
- **Grade:** **Fair**

## Overall verdict

PyForge PRD 在**产品定位**和**边界定义**方面表现出色——"uv 管依赖，PyForge 管项目"的 thesis 贯穿全文，竞品分析扎实，CLI 接口设计完整。但 PRD 在**可测试性**和**决策透明度**方面存在结构性缺口：FR 缺乏验收标准（critical），没有 Open Questions 章节，Success Metrics 不验证核心 thesis。Adversarial 评审还揭示了更深层的问题——没有考虑 uv 自身可能填补这个空白、索引文件的竞态条件、以及 Python 项目识别标准的缺失。PRD 足以进入架构设计，但在投入开发前必须补上 FR 级别的验收标准。

## Dimension verdicts

- Decision-readiness — adequate
- Substance over theater — strong
- Strategic coherence — adequate
- Done-ness clarity — thin
- Scope honesty — adequate
- Downstream usability — adequate
- Shape fit — strong

## Findings by severity

### Critical (2)

**[Rubric — Done-ness clarity]** 几乎所有 FR 缺少验收标准 (§3)
F1-F37 以 CLI 命令格式列出功能意图，但没有任何一个 FR 包含可验证的验收条件。下游 Epics/Stories 无从知道"做完"的边界。
*Fix:* 为每个 P0 FR 补充 1-2 个 Acceptance Criteria。

**[Adversarial]** 没有考虑 uv 的竞争反应 (§1, §9)
PRD 假设 uv 永远只做依赖管理。uv 已发展成生态系统（`uv run`, `uv build`, `uv publish`），完全可能在一年内增加项目索引功能，届时"空白地带"将不复存在。
*Fix:* 明确差异化策略，或承认此风险。

### High (6)

**[Rubric — Decision-readiness]** 没有 Open Questions 章节
PRD 没有任何"未决问题"的集中表达，隐藏了决策中的张力，使读者无法判断哪些是确定的、哪些是待定的。
*Fix:* 增加 Open Questions 章节，标记每项未决决策。

**[Rubric — Strategic coherence]** Success Metrics 不验证核心 thesis (§13)
指标测的是"多少人用了"而非"用得好不好"——下载量、Star 数、扫描速度都是活动指标，没有 NPS、留存、或用户粘性指标。
*Fix:* 增加 1-2 个产品价值指标。

**[Rubric — Done-ness clarity]** "详细信息""概览"等模糊表述 (§3)
F4、F18、F21 多处使用"详细信息""概览""分布"等未量化的名词。
*Fix:* 对每个模糊字段给出具体字段列表。

**[Rubric — Done-ness clarity]** "复用 uv"缺乏集成定义 (§9)
多处提到"调用 uv init""复用 uv"，但没有定义调用路径（CLI 子进程？Rust API？）。
*Fix:* 明确集成策略。

**[Adversarial]** 索引文件的竞态条件和数据一致性问题 (§7, §8)
`projects.json` 可能被多个终端会话同时修改，JSON Schema 校验和 `.bak` 备份不能完全解决竞态问题。
*Fix:* 使用文件锁或 SQLite。

**[Adversarial]** Web 看板是过度的分布式架构 (§3.5, §7)
对于核心场景，启动 HTTP 服务器远比 `pyforge list` 重。启动时间、运行模式、与 CLI 功能重叠等问题未处理。
*Fix:* 将 Web 看板降级为 P2，先确保核心 CLI 体验。

### Medium (6)

**[Rubric — Decision-readiness]** 技术选择仅有"或"无结论 (§7.1)
"tiny_http 或 axum"暴露了架构决策未完成。
*Fix:* 二选一并说明理由。

**[Rubric — Strategic coherence]** 没有 counter-metrics
"又多了一个要学的工具"是真实成本，未被承认。
*Fix:* 增加"学习成本"或"初次设置时间"作为负面指标。

**[Rubric — Downstream usability]** 缺少 Glossary
关键术语"子包""脚手架""跟踪"未定义。
*Fix:* 增加 Glossary 章节。

**[Rubric — Scope honesty]** 无 `[ASSUMPTION]` 标注
多处未验证的推断没有标记为假设。
*Fix:* 添加假设标注并建立 Assumptions Index。

**[Adversarial]** i18n 的优先级过高 (§3.7, §6)
7 个 FR 分配给 i18n 且标记为 P1，而核心功能仍在 P1/P2。
*Fix:* 将完整 i18n 框架降到 v1.1+。

**[Adversarial]** 没有离线/降级策略
模板获取方式（内置 vs 远程）、索引恢复流程等未定义。
*Fix:* 明确联网需求，确定模板分发策略。

### Low (3)

**[Rubric — Substance over theater]** NFR 中"安全性"和"可扩展性"缺少阈值 (§10)
*Fix:* 补充具体约束。

**[Adversarial]** 没有 Shell 补全的设计 (§4)
*Fix:* 在 CLI 设计阶段加入 Shell 补全支持。

**[Adversarial]** SKILL.md 分发机制假设过多 (附录 B)
自动写入 AI 工具配置属于侵入性操作。
*Fix:* 改为安装时提示 + `pyforge init-agent` 手动命令。

## Mechanical notes

- F-ID 连续（F1-F37），无断号或重复
- 没有 Glossary
- 没有 Assumptions Index
- 用户故事有角色名但无正式 UJ 编号
- 目录完整、跨章节引用一致

## Reviewer files

- `review-rubric.md`
- `review-adversarial.md`
