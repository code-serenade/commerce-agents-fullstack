# Post 8-131：Agent Chat 移动端真实导航与滚动治理

状态：done

## 本次目标

本轮不新增后端 API，也不伪造新能力，而是把已经存在的真实能力在 Agent Chat 前端壳层里更稳定地暴露出来：

- 历史会话来自 `AgentChatShell` 当前接入的真实 conversation 状态。
- 功能导航只跳转到新产品已有真实路由，例如能力介绍、问答、商品、活动、平台连接、交付资料、审批、工作区和能力包。
- 移动端默认不展示拥挤的左侧栏，而是使用同事框架里的 `SidebarShell` / `MobileBottomNav` 能力承接。
- 移动端弹出的历史 / 功能列表增加高度限制和滚动条，避免项目多时遮挡主问答区域。

## 修改范围

- `frontend/apps/web/src/shell/AgentChatShell.tsx`
  - 为 Agent Chat 壳层补充移动端底部导航。
  - 移动端入口包含：新建会话、最近历史、功能导航、管理与复盘。
  - 最近历史只取最近 5 个真实会话，不伪造历史记录。
  - 选择历史会话时调用真实 `chat.selectConversation`，并回到 `/commerce` 问答主界面。

- `frontend/packages/app-kit/src/components/app-shell/mobile-bottom-nav.tsx`
  - 为移动端二级弹出导航增加最大高度和 `overflow-y-auto`。
  - 多个历史会话或多个功能入口不会继续把页面往上遮挡。

## 产品内核变化

- 后端内核、Agent skill、SSE 和 API 本轮没有改动。
- 本轮只做“前端对齐已存在后端状态和真实路由”的装配。

## 用户可感知变化

- 手机或窄屏打开 Agent Chat 时，底部会出现清晰的四个入口：新建会话、历史会话、功能导航、管理与复盘。
- 用户可以在移动端直接切换最近 5 个真实历史会话，不再只能依赖桌面左侧栏。
- 功能导航和治理入口展开后，如果内容较多，会在弹层内部滚动，不会大面积遮挡聊天输入区和回答正文。
- 桌面端仍然使用原有左侧可折叠 `SidebarShell`，不增加第二套侧栏。

## 验证记录

- [x] `pnpm -C packages/app-kit typecheck`
- [x] `pnpm -C packages/app-kit rules`
- [x] `pnpm -C apps/web lint`
- [x] `pnpm -C apps/web test`
- [x] `pnpm -C apps/web build`
- [x] `pnpm -C apps/web rules`
- [x] `pnpm rules`

## 后续缺口

- 继续按后端优先原则迁移 skill 能力。
- 移动端历史会话目前只展示最近 5 个；后续如需要，可继续补“更多历史”入口，但必须仍然读取真实 conversation 数据。
