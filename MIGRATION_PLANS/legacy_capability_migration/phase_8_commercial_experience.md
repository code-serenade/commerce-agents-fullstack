# 8/8：完整商业化体验

状态：done

## 本次目标

把旧产品“电商多智能体/虚拟员工”的商业场景入口迁移到新产品框架里。后端先提供真实场景 playbook，前端再按后端契约展示并填充任务，避免继续用前端硬编码样例伪装能力。

## 本次完成

- 后端新增 `CommercePlaybookApi`。
- 后端新增 `GET /api/app/agent/playbooks`。
- 返回 6 个核心场景：
  - 新品冷启动
  - 投放复盘
  - 客诉危机
  - 选品定价
  - 素材优化
  - 日报复盘
- 每个场景包含：
  - 推荐回答策略
  - 所需上下文
  - 预期产出
  - 虚拟员工角色
  - 可直接进入问答链路的 prompt
- 前端问答输入区改为读取后端 playbook。
- 点击场景后自动填入任务，并切换到后端推荐策略。

## 产品内核变化

问答入口从“普通输入框 + 前端样例”升级为“后端商业场景目录 + 策略化任务入口”。这让用户一进入产品就能按电商岗位工作流启动虚拟员工，而不是只靠自己写一段普通问题。

## 后端变化

- 新增 `backend/crates/service/src/dto/commerce_playbook.rs`
- 新增 `backend/crates/service/src/api/commerce_playbook.rs`
- 新增 `backend/crates/web-server/src/handlers/app/commerce_playbook.rs`
- 新增 `backend/crates/web-server/src/routes/app/commerce_playbook.rs`
- 注册 service/web-server 模块和 app route。
- API 使用现有 App 用户鉴权边界，当前用户必须是 enabled。

## 前端变化

- `frontend/apps/web/src/api/commerce.ts` 新增 playbook API 类型和请求方法。
- `agent-chat` 新增 playbook domain、actions、hook。
- `AgentChatComposer` 删除原前端硬编码样例入口，改为展示后端返回的场景按钮。
- 页面 hook 拆出 command view-model，保持模块边界规则通过。

## 用户可感知变化

- 用户打开问答页后，会看到 6 个电商业务场景入口。
- 点击“新品冷启动”“投放复盘”等场景，会自动把完整任务写入输入框。
- 对应的回答策略会自动切换，例如复杂场景走多智能体协作，选品定价走学习解释，客诉危机走执行优先。
- 用户仍可继续选择商品上下文，发送后继续使用前面已经迁移的 RAG、工作区、质量检查、审批告警等链路。

## 验证记录

- [x] `cargo fmt`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `cargo test -p service commerce_playbook`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `git diff --check`
