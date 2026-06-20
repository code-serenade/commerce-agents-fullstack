# AI 电商搭档自然语言入口前端开发计划

## 需求来源

本需求来自当前聊天确认：客户反馈界面体感不好，且希望 AI 更像平等、懂电商、愿意一起推进问题的伙伴，而不是老师、专家报告或配置台。

## 用户体感目标

用户打开聊天页时应感受到：

- 可以直接说问题，不需要先懂 Agent、skill、角色、协作模式。
- 信息不完整也没关系，AI 会先帮忙判断，再问关键问题。
- 页面不像控制台，像一个能直接开口的对话入口。
- 高级能力存在，但默认不压迫用户。

## 本阶段范围

### 做

- 优化 `apps/web` 的 agent-chat 首屏、欢迎语、场景入口和设置显隐。
- 将“角色/策略/资料/过程”的强配置感弱化为渐进展开。
- 更新中英文 locale 文案，减少说教、老师感和系统术语。
- 保留现有 runtime、资料、过程能力，但默认让自然语言输入成为主入口。

### 不做

- 不新增后端 API。
- 不新增数据库。
- 不新增全局共享组件。
- 不做长期记忆 UI 的完整实现，只为后续设计保留方向。
- 不切分支。

## 目标层级

本次前端改动属于 `frontend/apps/web` 的复杂交互页面装配层。

已检查共享库存：

- `@workspace/ui-components` 已提供 Button、Textarea、Badge、Collapsible、Tabs、Sheet 等基础能力。
- `@workspace/app-kit` 已提供 shell、feedback、tooltip、file-upload 等应用能力。

本阶段不新增可复用共享组件，原因是改造内容只服务 `apps/web` 的 agent-chat 页面体验和业务文案。

## 预计修改文件

- `frontend/apps/web/src/pages/site/agent-chat/index.tsx`
- `frontend/apps/web/src/pages/site/agent-chat/components/agent-chat-thread.tsx`
- `frontend/apps/web/src/pages/site/agent-chat/components/agent-chat-composer.tsx`
- `frontend/apps/web/src/pages/site/agent-chat/components/agent-chat-composer-toolbar.tsx`
- `frontend/apps/web/src/pages/site/agent-chat/components/agent-chat-strategy-bar.tsx`
- `frontend/apps/web/src/pages/site/agent-chat/components/agent-chat-runtime-options-bar.tsx`
- `frontend/apps/web/src/pages/site/agent-chat/agent-chat-data.ts`
- `frontend/apps/web/lang/zhCN/messages.ts`
- `frontend/apps/web/lang/en/messages.ts`
- 必要时补充 `components/` 或 `view-model/` 的页面私有模块

## 交互设计

### 首屏

- 主输入框是第一视觉中心。
- 欢迎语强调“不完整也可以先发”。
- 展示 4-5 个自然场景入口：
  - 看看这个商品能不能卖
  - 帮我改标题/卖点
  - 帮我写客服回复
  - 帮我复盘数据
  - 帮我把方案说得更靠谱

### 高级能力

- 回答风格保留，但使用用户语言：
  - 直接点
  - 详细一点
  - 一起想
- 角色、多岗位协作、资料选择、过程面板默认弱化或收起。
- 过程面板入口改成“看看我是怎么判断的”一类用户语言。

### 文案原则

- 避免“请选择”“配置”“策略”“角色执行”等系统语。
- 避免“教练”“导师”“老师式点评”的默认口吻。
- 使用“我们先看”“我先帮你判断”“不完整也没关系”等低压力表达。

## 验收标准

- 新用户能在首屏直接发问，不需要先配置。
- 空状态不再像功能说明页。
- 第一屏没有过多问题、角色、策略和过程信息压迫用户。
- 场景入口点击后能直接进入自然语言问题。
- 移动端不出现按钮文字溢出、控件重叠或输入框被挤压。

## 验证计划

- `pnpm -C apps/web lint`
- `pnpm -C apps/web test`
- `pnpm -C apps/web rules`
- `pnpm -C apps/web build`
- 根目录或 workspace 必要规则检查按项目脚本执行。

