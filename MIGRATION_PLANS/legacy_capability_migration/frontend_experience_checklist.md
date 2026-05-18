# 前端体验迁移检查清单

## 前端适配前置条件

前端不是本迁移的起点。每项前端体验开始前，必须确认后端已经提供稳定契约：

- [ ] 对应后端能力已经完成或明确标记为 deferred。
- [ ] 对应 API/SSE event 已写入 `API_DOCS/`。
- [ ] DTO、错误码、权限规则已经稳定。
- [ ] 后端已通过 cargo 验证、agent smoke 或真实 SSE/HTTP 请求。
- [ ] 前端不会伪造 route、worker、skill、quality、approval、artifact 状态。

## 问答主体验

- [x] 第一屏突出电商多智能体问答。
- [x] 回答策略选择可用，并传给后端 metadata。
- [x] 策略切换产生的差异来自后端 route/worker/SSE event。
- [x] 商品上下文选择可用，并传给后端 metadata。
- [x] 商业场景入口来自后端 playbook。
- [x] 用户反馈入口可对真实 workspace run 提交评分和评论。
- [x] 用户问题、助手回答、过程面板布局清晰。
- [x] 未登录状态提示清楚。
- [x] 发送中、停止、错误、超时状态完整。

## 左右侧折叠

- [x] 左侧历史/导航可折叠。
- [x] 右侧虚拟员工工作区可折叠。
- [x] 移动端默认合理收起。
- [x] 折叠状态不遮挡输入区。

## Runtime 展示

- [ ] route decision 可视化，数据来自后端 SSE。
- [ ] worker started/finished 可视化，数据来自后端 SSE。
- [ ] task graph 可视化，数据来自后端 SSE。
- [x] skill requested/result 可视化，数据来自后端 SSE。
- [x] context loaded 可视化，数据来自后端 SSE。
- [x] workspace run/task 沉淀状态可视化，数据来自后端 SSE。
- [x] quality check 可视化，数据来自后端 SSE/API。
- [x] approval required 可视化，数据来自后端 SSE/API。
- [x] workspace artifact 可视化，数据来自后端 SSE。
- [x] RAG 来源和降级原因可视化，数据来自后端 SSE。
- [x] 用户反馈提交，数据写入后端 learning record。

## 业务页面联动

- [ ] 产品页可带入问答，后端能读取真实产品上下文。
- [ ] 活动页可让 Agent 优化/复盘，后端已有对应业务 API。
- [x] 工作区页刷新后展示 agent run 和 artifact 列表，后端已有 workspace run/artifact。
- [x] 审批页承接高风险工具，后端已有 approval record。
- [x] 审批页可展示平台执行 readiness，工作区执行建议可展示真实失败原因。
- [x] 平台连接页可展示平台执行 adapter 能力目录和不可执行原因。
- [x] 能力包页可同步旧产品内置 domain/role/skill/capability 治理记录。
- [ ] 历史会话可恢复上下文，后端已有 conversation/session 数据。

## 设计原则

- [x] 不新增第二套 UI 基础组件。
- [x] 优先使用 `@workspace/app-kit`。
- [x] 优先使用 `@workspace/ui-components`。
- [x] app 本地只做业务装配。
- [ ] 页面文案不写功能说明式长篇教程。
