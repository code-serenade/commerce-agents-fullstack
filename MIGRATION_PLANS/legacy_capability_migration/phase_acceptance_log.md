# 阶段验收记录

## 总门禁

每个阶段必须先通过后端门禁，再进入前端适配。

后端门禁：

- [ ] 后端能力边界确认完成。
- [ ] 涉及 API、DB、SSE、权限、错误码或 LLM 调用时，后端设计文档完成。
- [ ] Rust agent/service/web-server 实现完成。
- [ ] `API_DOCS/` 已同步。
- [ ] `cargo fmt`、`cargo check`、`cargo clippy`、相关测试或 smoke 已通过。
- [ ] 真实 SSE/HTTP 请求验证完成。

前端适配门禁：

- [ ] 对应后端 API/SSE 契约已稳定。
- [ ] 前端只消费真实后端数据和事件。
- [ ] 未接入的后端能力不做假流程展示。
- [ ] `pnpm rules` 和相关前端验证已通过。

## Phase 0：迁移台账与能力核对

状态：pending

验收项：

- [ ] 旧 46 skill 全部映射完成。
- [ ] 新 93 staged spec 核对完成。
- [ ] 每个 skill 标记 `native_ready` / `adapter_required` / `approval_required` / `missing`。
- [ ] 角色矩阵核对完成。

## Phase 1：问答策略与路由增强

状态：done

验收项：

- [x] 后端：execution/learning/collab 策略进入 request metadata。
- [x] 后端：Commander 能按策略影响路由。
- [x] 后端：SSE 输出真实 route decision / strategy applied。
- [x] 简单问答走轻流程。
- [x] 复杂任务触发多角色。
- [x] 前端：策略切换只展示后端真实行为差异。

## Phase 2：Skill 执行与 SSE 可视化

状态：done

验收项：

- [x] 后端：`skill_requested` 事件。
- [x] 后端：`skill_result` 事件。
- [x] 后端：失败、deferred、approval_required 状态结构化。
- [x] 后端：至少 6 个代表 skill 可真实执行或明确 deferred。
- [x] 前端：运行详情卡和时间线接真实 skill 事件。

## Phase 3：产品知识与上下文绑定

状态：done

验收项：

- [x] 后端：SSE request 携带 context refs。
- [x] 后端：读取真实产品知识。
- [x] 后端：权限、空数据和超长上下文降级规则明确。
- [x] 回答引用真实产品信息。
- [x] 前端：可选择产品上下文并展示引用来源。

## Phase 4：工作区沉淀与交付物

状态：done

验收项：

- [x] 后端：Agent run 关联 workspace run。
- [x] 后端：Worker report 可沉淀任务。
- [x] 后端：artifact 创建、查询、权限闭环。
- [x] 前端：问答右侧面板可展示后端 `workspace_artifact_created` 交付物事件。
- [x] 前端：完整工作区页面可刷新后查看 artifact 列表和详情。

当前说明：

- 已完成 workspace/run/task/artifact 后端闭环。
- Agent SSE 正常完成时会创建 `final_report` 交付物，并发出 `workspace_artifact_created`。
- 完整工作区页面已接入后端 artifact 查询和详情接口，刷新后可查看交付物列表、预览正文、来源、格式、存储、运行 ID 和校验状态。

## Phase 5：质量、信任、学习闭环

状态：done

验收项：

- [x] 后端：质量评分结构可用。
- [x] 后端：用户反馈可保存。
- [x] 后端：trust/learning 可影响后续回答。
- [x] 前端：质量评分展示真实数据。
- [x] 前端：用户反馈入口已展示并可提交。

当前说明：

- 已完成质量检查、信任分、学习记录三张表和后端闭环。
- Agent SSE 正常完成时会自动质量检查，并发出 `quality_checked`。
- 前端问答页右侧过程区已展示质量分、信任分和学习记录数量。
- 前端问答页右侧工作台已接入真实运行反馈入口，提交后写入后端学习记录。

## Phase 6：审批、告警、平台写操作

状态：done

验收项：

- [x] 后端：平台写操作默认 pending approval。
- [x] 后端：审批记录可查。
- [x] 后端：告警可推送。
- [x] 前端：审批和告警只展示后端真实记录。

当前说明：

- Agent SSE 收到 `skill_result.status=pending_approval` 或 `approval_required=true` 时，会自动创建或复用工具审批记录。
- 同步写入 `tool_approval.create` 审计日志，并创建或复用 `agent_approval_required` 告警。
- 前端问答页右侧时间线和工作区卡片已消费真实 `approval_required` 事件。
- 本阶段只完成审批记录、审计和告警，不执行平台写操作。

## Phase 7：搜索/RAG/外部数据

状态：done

验收项：

- [x] 后端：搜索结果有来源。
- [x] 后端：产品知识可检索。
- [x] 后端：外部不可用可降级。
- [x] 前端：来源和降级原因可见。

当前说明：

- 已完成本地 RAG 检索，来源来自当前用户自己的 active 商品和 active 商品知识。
- Agent SSE 会发出 `rag_context_loaded`，并把检索摘要注入 agent metadata。
- 外部搜索适配器未接入时不伪造结果，前端展示明确降级原因。

## Phase 8：完整商业化体验

状态：done

验收项：

- [x] 新品冷启动场景全通。
- [x] 投放复盘场景全通。
- [x] 客诉危机场景全通。
- [x] 选品定价场景全通。
- [x] 素材优化场景全通。
- [x] 日报复盘场景全通。

当前说明：

- 后端已提供 6 个商业场景 playbook，并走 App 用户鉴权。
- 前端问答输入区已改为读取后端 playbook；点击后自动填入任务并切换推荐策略。
- 本阶段完成的是商业场景入口和问答链路适配，不新增用户自定义 playbook 数据库能力。

## Phase 9：登录态恢复与旧产品内置能力包治理同步

状态：done

验收项：

- [x] 后端：旧产品内置 domain/role/skill manifest 可同步到当前用户能力包治理目录。
- [x] 后端：派生 capability 记录可用于能力矩阵和依赖校验。
- [x] 后端：批量同步使用事务，重复同步不重复创建。
- [x] API：同步接口写入 `API_DOCS/app.md`。
- [x] 前端：能力包治理页提供真实同步入口。
- [x] 前端：登录态恢复后刷新受保护页面不再误跳登录。

当前说明：

- 本阶段同步的是治理目录，不动态加载 Agent runtime catalog。
- 旧产品 role manifest 的旧 skill 函数名保存为 metadata，不作为新治理依赖，避免校验误报。
- 真实 HTTP smoke 已确认同步 52 条记录，`builtin.ops` 和 `role.ops-strategist` 校验通过。

## Phase 10：Agent 代表 Skill 自动补偿触发

状态：done

验收项：

- [x] 后端：模型显式 `skill_requests` 优先，不被自动补偿覆盖。
- [x] 后端：模型没有输出 `skill_requests` 时，可按岗位和任务自动补偿代表 skill。
- [x] 后端：自动补偿仍通过角色白名单和 `SkillExecutorSet`，不绕过权限。
- [x] 后端：默认业务问题不会被过宽的 chat bypass 吃掉，明显岗位任务会进入 task graph。
- [x] 后端：stream 级别真实发出 `skill_requested` / `skill_result`。
- [x] API：`API_DOCS/sse.md` 已说明自动补偿触发语义。
- [x] 前端：不新增假展示，沿用真实 SSE 事件展示 skill 调用。

当前说明：

- 已完成 ops/data/accounting/service/creative/web/design/engineering 的第一层代表 skill 自动补偿。
- 已修复默认漏斗问题只走 chat 的断点；真实云端 Ollama smoke 已确认默认漏斗问题进入 data+ops 任务图并发出 skill 事件。
- 该阶段提升的是“能稳定触发旧 skill 生态入口”，不是把所有旧 skill 都完整接入外部搜索、平台执行、内容生成或导出产物。
- 高风险平台写操作仍不会自动执行；需要审批的 skill 继续进入 `pending_approval`。

## Phase 11：问答内核调度说明与轻问答兼容

状态：done

验收项：

- [x] 后端：`chat` 角色轻问答不强制 `findings/recommendations`。
- [x] 后端：短确认、能力说明、参与角色说明不再因复杂质量规则误判失败。
- [x] 后端：运营、数据、内容虚拟员工分工问题可稳定进入 `ops -> data/creative` 任务图。
- [x] 后端：中文提示词约束减少英文混入。
- [x] 前端：不新增接口，沿用现有 SSE 和任务图展示。

当前说明：

- 本阶段修复的是问答内核和后端任务图调度，不新增数据库表。
- `chat` 角色仍需要有摘要内容，但不再被复杂业务岗位的质量条款误伤。
- 角色分工类问题可由配置任务图稳定编排运营、数据、内容创意三个岗位。

## Phase 12：创意内容类 Skill Native 迁移

状态：done

验收项：

- [x] 后端：短视频脚本、种草文案、IP 定位、内容排期、趋势捕捉、直播脚本、产品软文均可 native success。
- [x] 后端：creative 岗位自动补偿能按用户意图命中不同内容类 skill。
- [x] 后端：结果通过既有 `SkillExecutorSet`、角色白名单和 SSE skill 事件链路输出。
- [x] 前端：不新增假展示，沿用现有 `skill_requested` / `skill_result` / 最终回答。

当前说明：

- 本阶段把旧产品创意岗位的内容生成能力迁到新产品 Rust Agent runtime，当前以本地结构化文本产物为主。
- 结果包括脚本结构、种草正文草稿、IP 人设、内容日历、趋势方向、直播阶段话术、产品软文结构。
- 外部实时趋势、真实商品详情自动加载、图片/视频/DOCX 文件生成仍是后续阶段。

## Post 8-5：显式岗位选择优先级与前端运行配置稳定性

状态：done

验收项：

- [x] 后端：`manual/single + primaryRole/selectedRoles` 中包含非 `chat` 岗位时，短答旁路不会吞掉显式岗位选择。
- [x] 后端：没有显式非 `chat` 岗位选择的短答请求仍可走 `chat_bypass`，保持轻问答体验。
- [x] 后端：`route_decided` 能在回归测试中返回 `task_graph`、主岗位和支持岗位。
- [x] 前端：多岗模式下主岗位和参与岗位分开控制，发送给后端的运行选择更稳定。
- [x] API：`API_DOCS/sse.md` 已说明显式岗位选择优先于 `chat_bypass`。
- [x] 真实 HTTP/Edge smoke：业务漏斗问题按 `data + ops` 进入任务图并触发 native skill，页面真实请求体包含主岗位 `data` 和参与岗位 `data/ops`。

当前说明：

- 本阶段是问答调度契约补强，不新增数据库表，也不新增 HTTP/SSE 字段。
- 前端只适配后端运行契约，不伪造 skill、任务图或岗位执行结果。
- 本阶段不包含外部搜索、附件、导出报告或平台执行 adapter 迁移。

## Post 8-6：设计与 Web 增长 Skill Native 迁移

状态：done

验收项：

- [x] 后端：`design_main_image`、`design_detail_page`、`design_color_scheme`、`design_material_spec`、`design_campaign_poster` 返回 native success。
- [x] 后端：`web_seo_optimize`、`web_title_generator`、`web_page_conversion`、`web_store_design`、`web_keyword_research`、`web_product_description` 返回 native success。
- [x] 后端：design/web 岗位自动补偿能按用户意图命中不同设计和增长 skill。
- [x] 后端：结果通过既有 `SkillExecutorSet`、角色白名单和 SSE skill 事件链路输出。
- [x] 验证：agent skill 执行测试通过。
- [x] 验证：autosuggest 规则测试通过。
- [x] 验证：`cargo test -p agent` 通过。
- [x] 验证：`cargo check -p agent` 通过。
- [x] 验证：`cargo clippy -p agent` 通过。

当前说明：

- 本阶段把旧产品设计/Web 岗位的内容方案类能力迁到新产品 Rust Agent runtime，当前以本地结构化文本产物为主。
- 结果包括设计 brief、平台规格、配色 HEX、SEO 提排路径、关键词矩阵、标题候选、详情文案和页面转化 AB 测试清单。
- 图片/长图文件生成、实时搜索量、CPC、店铺真实指标和素材文件化仍是后续阶段。

## Post 8-7：客服处理 Skill Native 迁移

状态：done

验收项：

- [x] 后端：`service_ticket_handler` 返回 native success，输出工单优先级、SLA、处理策略、客服话术和升级条件。
- [x] 后端：`service_faq_playbook` 返回 native success，输出 FAQ 简短版、详细版和处理指引。
- [x] 后端：`service_escalation_flow` 返回 native success，输出严重度、升级路径、主管话术、赔偿建议和复核模板。
- [x] 后端：`service_dsr_improvement` 不再返回 deferred dependencies，输出 native DSR 改善方案。
- [x] 后端：service 岗位自动补偿能按用户意图命中工单、FAQ、升级、退换货、DSR、NPS等客服 skill。
- [x] 后端：结果通过既有 `SkillExecutorSet`、角色白名单和 SSE skill 事件链路输出。
- [x] 前端：不新增假展示，沿用现有 `skill_requested` / `skill_result` / 最终回答。
- [x] 验证：agent skill 执行测试通过。
- [x] 验证：autosuggest 规则测试通过。
- [x] 验证：`cargo test -p agent` 通过。
- [x] 验证：`cargo check -p agent` 通过。
- [x] 验证：`cargo clippy -p agent` 通过。
- [x] 验证：`cargo check` 通过。
- [x] 验证：`cargo clippy` 通过。

当前说明：

- 本阶段把旧产品客服岗位的工单、FAQ和升级处理能力迁到新产品 Rust Agent runtime，当前以本地结构化 SOP 和话术产物为主。
- 结果包括优先级、SLA、客服话术、升级条件、FAQ答案、处理指引、主管接管话术、赔偿建议和 DSR 改善动作。
- 实时平台政策、店铺真实售后规则、商品画像自动读取和客服 SOP 文档导出仍是后续阶段。

## Post 8-8：外部搜索 Adapter 契约化

状态：done

验收项：

- [x] 后端：`search_trends` 返回统一 `adapter_required` 降级契约，不伪造 sources。
- [x] 后端：`search_competitor` 返回统一 `adapter_required` 降级契约，不伪造 sources。
- [x] 后端：`search_market_info` 返回统一 `adapter_required` 降级契约，不伪造 sources。
- [x] 后端：`search_product_reviews` 返回统一 `adapter_required` 降级契约，不伪造 sources。
- [x] 后端：`search_platform_policy` 返回统一 `adapter_required` 降级契约，不伪造 sources。
- [x] 后端：`search_industry_benchmarks` 返回统一 `adapter_required` 降级契约，不伪造 sources。
- [x] 后端：六个 search skill 均保留 `status=deferred`，runtime metadata 仍包含 search/LLM adapter requirements。
- [x] 后端：输出包含 `query`、`data.items=[]`、`sources=[]`、`provenance.source_count=0`、`confidence=0.0`、`degraded=true`。
- [x] 前端：本阶段不新增假展示，继续沿用真实 `skill_result` 状态。
- [x] 验证：搜索契约测试通过。

当前说明：

- 本阶段没有接入真实互联网搜索，不声明 provider，不返回 `_search_engine`。
- 本阶段解决的是新产品后端搜索能力的稳定输出契约，给后续真实 search provider/API/SSE 展示铺路。
- 外部实时搜索、来源展示、置信度展示和前端搜索来源面板仍是后续阶段。

## Post 8-9：Rust 外部搜索 Adapter API

状态：done

验收项：

- [x] 后端：新增 `GET /api/app/external-search/status`，返回 adapter 启用和配置状态。
- [x] 后端：新增 `GET /api/app/external-search/search`，查询参数包含 `q/topic/days/maxResults/bypassCache`。
- [x] 后端：未配置 adapter 时返回 `status=degraded`，不伪造来源。
- [x] 后端：provider payload 可归一化为 `data.items`、`provenance`、`confidence`。
- [x] 后端：当前 App 用户鉴权与 enabled 校验沿用现有 `app_users`。
- [x] API：`API_DOCS/app.md` 已同步新增接口。
- [x] 配置：`services-example.toml` 新增 `[external_search]` 示例并默认关闭。
- [x] 前端：本阶段不新增假展示。

当前说明：

- 本阶段完成的是 Rust web/service 层的搜索 adapter API 入口，不是 Agent skill 的 live 调用。
- 当前 `services.toml` 未配置 `[external_search]` 时，产品启动不受影响，接口会返回明确 degraded。
- 下一阶段可以把 Agent `search_*` skill 接入该 adapter 契约，或先做前端来源展示。

## Post 8-10：Agent 搜索 Skill 接入外部搜索 Adapter

状态：done

验收项：

- [x] 后端：Agent runtime 可接收 `[external_search]` 配置。
- [x] 后端：worker skill executor 把搜索配置传给 `search_*` skill。
- [x] 后端：配置完整时，搜索 skill 会请求 provider，并把 provider payload 归一化为 `sources`、`data.items`、`provenance`、`confidence`。
- [x] 后端：provider 未配置、请求失败或无来源时返回降级状态，不伪造来源。
- [x] 后端：搜索类 skill 在角色提示目录中标记为 `adapter`，不再只是静态 `deferred`。
- [x] API/SSE：不新增路由，`API_DOCS/sse.md` 已同步搜索类 `skill_result` 语义。
- [x] 前端：本阶段不改前端，不新增假搜索展示。

## Post 8-11：Agent 搜索证据沉淀为 Workspace Artifact

状态：done

验收项：

- [x] Agent：`skill_result` 事件增加可选结构化 `output` 字段。
- [x] 后端 service：新增 `record_agent_search_evidence`，复用现有 `workspace_artifacts` 表写入 `artifactType=search_evidence`。
- [x] 后端 service：只接受 `search_* + status=success + executionMode=adapter + output.status=live + sources 非空` 的真实搜索结果。
- [x] Web-server：SSE 收到 live 搜索结果后自动写入 `search_evidence`，并发送 `workspace_artifact_created`。
- [x] Web-server：搜索证据写入失败时返回 `workspace_run_persistence_error`，不截断原始回答。
- [x] API/SSE：`API_DOCS/sse.md` 和 `API_DOCS/app.md` 已同步 `search_evidence` artifact 契约。
- [x] 前端：本阶段不改前端，不新增假来源展示。
- [x] 验证：`cargo test -p service search_evidence_artifact` 通过。
- [x] 验证：`cargo test -p web-server skill_result_output_is_camelized_for_sse_consumers` 通过。
- [x] 验证：`cargo test -p agent` 通过。
- [x] 验证：`cargo check -p agent/service/web-server` 通过。
- [x] 验证：`cargo check`、`cargo clippy`、`git diff --check` 通过。

当前说明：

- 本阶段解决的是“真实搜索结果可沉淀、可追溯、可被工作区复用”。
- 如果 `[external_search]` 未配置或 provider 降级，不会创建搜索证据交付物。
- 前端后续可基于 `workspace_artifact_created` 或 artifact 查询接口，把 `search_evidence` 展示为来源面板。
