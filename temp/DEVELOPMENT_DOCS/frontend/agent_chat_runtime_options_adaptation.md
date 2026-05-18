# Agent Chat Runtime Options 前端适配说明

## 目标

前端不再硬编码旧产品问答策略能力，而是从后端 `GET /api/app/agent/runtime-options` 读取运行选项，并在发送 SSE 问答时把用户选择作为 metadata 传回后端。后端仍是能力根，前端只做展示和参数传递。

## 页面范围

页面：`frontend/apps/web/src/pages/site/agent-chat/`

职责：

- 页面 hook 负责读取 runtime options 和维护当前选择。
- API adapter 负责调用 runtime options 接口。
- composer 内展示模型、学习层级和角色选择。
- send-message 只发送结构化 metadata，不再通过 prompt 拼接伪造策略。

## 映射关系

- 模型选择：`metadata.model.name`
- 策略选择：`metadata.strategy.id`
- 回答模式：`metadata.strategy.responseMode`
- 学习层级：`metadata.strategy.learningLevel`
- 协作模式：`metadata.strategy.collaborationMode`
- 主角色：`metadata.strategy.primaryRole`
- 已选角色：`metadata.strategy.selectedRoles`

## 用户可见变化

用户在问答输入区可以看到后端当前支持的模型和虚拟员工角色。切换这些选项后，本轮问答会真实影响后端 Agent runtime，而不是只改变前端文案。

## 降级策略

- runtime options 未加载成功时，继续使用与后端语义一致的默认策略：教学模式、自动分配、中等讲解深度，保证基础问答不被阻断。
- 如果后端没有返回模型，前端不显示模型选择。
- 如果没有返回角色，前端不显示角色选择。

## 验证

- `pnpm -C apps/web test`
- `pnpm -C apps/web rules`
- `pnpm -C apps/web lint`
- `pnpm -C apps/web build`
