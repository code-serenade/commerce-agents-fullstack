# Post 8-52：演示可用前端对齐

状态：done

## 目标

今天需要向公司汇报产品，本轮优先让前端围绕已经完成的后端能力变得可用、可讲清楚、可演示。

本轮不补新的后端能力，不新增第二套 UI 框架，不把尚未接入的真实平台 adapter 伪装成已成功执行。

## 本轮修改

### 问答工作台

- 问答页默认展示右侧“虚拟员工工作区”。
- 无 runtime event 时仍展示现有 idle workbench，让第一屏不是普通聊天框。
- 右侧面板继续使用现有折叠按钮和收起把手，保持 Kimi Agent 集群式的主问答区 + 过程区体验。

### 审批与执行结果

- 审批列表新增“后端执行结果”列。
- 展示后端返回的 `executionStatus`、`adapterStatus`、`retryable`、`providerResult` 和 `adapterRequest.idempotencyKey`。
- 当前真实平台 adapter 未接入时，页面会展示后端返回的失败边界，不显示假成功。

### 平台连接

- 平台执行能力目录补充展示 `requiredParams`、`idempotencyKeyFields` 和当前是否可直接执行。
- 平台页文案对齐后端现状：已有 adapter catalog、preflight/readiness、adapter request/outcome，但真实平台写 adapter 仍待接入。

### 前端框架边界

- 只修改 `apps/web` 页面装配层和语言文案。
- 继续复用 `@workspace/app-kit`、`@workspace/ui-components` 和 web 内已有 commerce 页面组件。
- 没有改 `frontend/packages/*`，没有新增通用组件库。

## 用户可感知变化

- 用户进入“电商多智能体”问答页时，能直接看到中间问答区和右侧虚拟员工工作区。
- 用户可以折叠/展开右侧工作区，主回答区不会被过程信息长期挤占。
- 汇报平台连接页时，可以说明后端已经有平台执行治理边界、必填参数、幂等字段和不可执行原因。
- 汇报审批页时，可以说明审批通过后会进入后端执行 outcome；当前真实平台 adapter 未接入会明确显示失败边界，而不是前端坏了。

## 验证

- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`

## 后续缺口

- 真实平台写 adapter 仍未实现，后续应接入 provider registry / adapter trait 后继续走审批、执行队列和审计闭环。
- 问答真实 LLM、SSE、workspace artifact 全链路仍需要在浏览器中继续做演示前 smoke。
- 左侧历史/导航折叠和更完整的 Kimi Agent 集群式体验，还可以作为下一轮前端体验整合继续推进。
