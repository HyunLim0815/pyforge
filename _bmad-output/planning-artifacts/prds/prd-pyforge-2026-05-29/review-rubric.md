# PRD Quality Review — PyForge

## Overall verdict

PyForge 的 PRD 在**定位清晰度**和**边界定义**方面表现出色——"不做依赖管理，只做项目管理"的 thesis 明确且一致地贯穿全文。竞争格局分析和 CLI 接口设计是最大亮点。但 PRD 在**可测试性**和**决策透明度**方面存在结构性缺口：FR 缺乏验收标准，没有 Open Questions/Assumptions 索引，NFR 部分在"可靠性/可观测性"维度几乎为空白。这是一个 **Good** 的 PRD——足以进入架构设计，但在投入开发前需要补上验收标准。

## Decision-readiness — adequate

PRD 在定位决策上非常果断：第 9 节的"PyForge 明确不做"是清晰的 negative space 声明，第 1 节的竞品分析有数据支撑。优先级（P0/P1/P2）是显式的开发排序信号。

但决策背后看不到**权衡**的痕迹：
- 为何选择 `tiny_http` 或 `axum` 作为 Web 服务（§7.1）？这是两个有截然不同 trade-off 的选项（轻量 vs 功能丰富），"或"字表明未做决策
- Python 版本扫描是用什么机制实现的（§3.5、§8.1 提到了 `python_version`）？是调用 `python --version` 还是解析 `pyproject.toml`？这是实现层面的关键分歧
- 没有 Open Questions 章节——一个 v1 产品不可能没有任何未决问题

### Findings
- **[high]** 没有 Open Questions 章节 — PRD 没有任何"未决问题"的集中表达。这隐藏了决策中的张力，让读者（架构师/开发者）无法判断哪些是确定的、哪些是待定的。*Fix:* 增加 Open Questions 章节，标记每项未决决策、Owner 和截止时间。
- **[medium]** 技术选择仅有"或"无结论 — §7.1 中 "tiny_http 或 axum" 暴露了架构决策未完成。*Fix:* 二选一并说明理由，或将此明确列为 Open Question。
- **[medium]** Python 版本检测机制未定义 — §8.1 展示了 `python_version` 字段，但未说明如何获取该值。*Fix:* 明确扫描策略（解析 `pyproject.toml` → 回退到 `python --version` 猜测？）。

## Substance over theater — strong

PRD 几乎没有"填充物"：
- **Persona theater (无)** — 6 个用户角色各具差异化场景，无冗余
- **Innovation theater (无)** — 竞争分析基于真实空白，定位原创性真实可信
- **NFR theater (部分)** — 大部分 NFR 有具体阈值（1000 目录 < 3s，< 8MB），但"安全性"（§10）和"可扩展性"（§10）缺乏产品特定的指标
- **Vision theater (无)** — "Python 项目管家"具体、可理解、不可迁移

### Findings
- **[low]** NFR 中"安全性"和"可扩展性"缺少阈值 — §10 中这两项只有定性描述，没有量化指标。*Fix:* 为"安全性"补充具体约束（如"仅接受本地请求，无认证要求"），为"可扩展性"明确扩展维度和上限。

## Strategic coherence — adequate

PRD 的核心 thesis 明确：**uv 管依赖，PyForge 管项目**。特征演进（track → scan → info → new → web →协作）遵循了"先有索引、再有操作、后有可视化"的自然顺序。

但存在两个结构性问题：
1. **Success Metrics（§13）不直接验证 thesis** — 下载量、Star 数、扫描速度等是活动指标，而非 thesis 有效性指标。没有指标回答"PyForge 是否真的让开发者更好地管理了项目？"
2. **没有 counter-metrics** — 增加"项目管家"层意味着额外的心理负担（又多了一个工具要学），这个成本没有被承认

MVP 范围策略是合理的"platform-first"（先建索引基础设施，再叠加体验功能）。

### Findings
- **[high]** Success Metrics 不验证核心 thesis — §13 的指标测的是"多少人用了"而非"用得好不好"。没有 NPS、用户留存、或"每周主动使用次数"这类产品粘性指标。*Fix:* 增加 1-2 个产品价值指标，如"周活跃用户平均查看项目数"或"新用户 7 天内创建项目数"。
- **[medium]** 没有 counter-metrics — "又多了一个要学的工具"是真实成本。*Fix:* 在 §9 或 §13 中增加 counter-metric："与 uv 命令行共存的学习成本"或"初次设置时间"作为负面指标追踪。

## Done-ness clarity — thin

这是 PRD 最薄弱的维度。F1-F37 全部是**功能意图描述**而非**可验收的功能定义**：

- F4（查看项目详细信息）：什么信息？完整清单未定义
- F6（输出项目路径）：唯一输出就是路径？还是包含项目名？
- F18（状态概览）："概览"包含哪些字段？git 状态、最近修改时间、还是依赖状态？
- F19（依赖更新）："复用 uv"不构成验收标准——是调用 `uv outdated` 并解析输出？还是借助 `uv lock` 对比？
- F21（仪表盘）："版本分布""工具链分布"——图表类型？交互方式？
- Web API 端点（§7.3）较好，但 `/api/open`（POST）的参数结构未定义

相比之下，竞品对比矩阵（附录 A）清晰可验证。

### Findings
- **[critical]** 几乎所有 FR 缺少验收标准 — F1-F37 以 CLI 命令格式列出功能意图，但没有一个 FR 包含"给定 X，执行 Y，应得到 Z"的验收条件。下游 Epics/Stories 无从知道"做完"的边界。*Fix:* 为每个 P0 FR 补充 1-2 个 Acceptance Criteria。
- **[high]** "详细信息""概览"等模糊表述 — F4、F18、F21 多处使用"详细信息""概览""分布"等未量化的名词。*Fix:* 对每个模糊字段给出具体字段列表（如 F4 输出：名称、路径、Python 版本、最后修改时间、Git 状态、依赖数、标签）。
- **[high]** "复用 uv"缺乏集成定义 — §9 中多处提到"调用 uv init"，§3.4 F19 "复用 uv"——但没有定义调用路径（CLI 子进程？Rust API？）。*Fix:* 明确集成策略：采用 CLI 子进程调用 + 解析 stdout/stderr。

## Scope honesty — adequate

亮点：§9 的"PyForge 明确不做"是教科书级的 negative space。b 优先级系统（P0/P1/P2）诚实地区分了"现在就要"和"以后再说"。

问题：
- **没有 `[ASSUMPTION]` 标注** — 任何推断（如"用户希望在 Web 看板中一键打开 VS Code"）都没有标记为假设
- **没有 `[NON-GOAL for MVP]` 标注** — v0.1 到 v1.0 的范围割裂清晰（§12），但 FR 级别没有标记 MVP cut
- **没有 Open Questions 章节**

### Findings
- **[medium]** 无 `[ASSUMPTION]` 标注 — PRD 包含多处未验证的推断（如用户需要中英文切换、社区会贡献语言包），但没有标记为假设。*Fix:* 在 §3 和 §6 中添加 `[ASSUMPTION: ...]` 标注，并在文档末尾建立 Assumptions Index。
- **[low]** 无 MVP 范围标记 — §12 的版本规划已经定义了各版本的范围，但 FR 级别没有 `[NON-GOAL for v0.1]` 标记。*Fix:* 在 P1/P2 功能后添加 `[NON-GOAL for MVP]` 标注以减少歧义。

## Downstream usability — adequate

PRD 结构清晰、目录完整，每个章节可独立阅读。F 编号连续（F1-F37），跨章节引用一致。

主要不足：
- **没有 Glossary** — "脚手架""子包""跟踪""索引"等术语在多个章节出现，但从未正式定义。下游团队可能产生歧义（"子包"是指 namespace package 还是 regular package？）
- **用户故事没有正式 ID** — 8 个用户故事（§11）使用数字列表而非 UJ1-UJ8 编号，无法在 FR 中精确引用
- 跨引用采用标题链接而非术语索引，在单独提取章节时可能丢失上下文

### Findings
- **[medium]** 缺少 Glossary — 关键术语"子包""脚手架""跟踪"未定义，下游 UX/Architecture 可能产生歧义。*Fix:* 增加 Glossary 章节，定义所有领域名词。
- **[low]** 用户故事无 ID — §11 的 8 个故事使用无序数字列表，无法在 FR 或架构讨论中精确引用。*Fix:* 为每个用户故事分配 UJ1-UJ8 ID。

## Shape fit — strong

PyForge 是一个**开发者工具**（CLI + Web），当前采用**能力规格**（capability spec）的形态非常合适：特征表格为主、用户故事为辅助示例。

用户故事数量（8 个）对于 CLI 工具合理，没有过度形式化。非功能需求（§10）的量化程度对开发者工具来说恰到好处。

### Findings
无。

## Mechanical notes

- 用户故事有角色名（小明、小红、小李等）但缺乏实质性人物背景——对于 CLI 工具这不构成问题，因为用户故事在本文中只是示例而非负载设计决策
- 没有 Assumptions Index
- 没有 Glossary
- F-ID 连续（F1-F37），无断号或重复
- 没有跨章节的标题编号冲突
