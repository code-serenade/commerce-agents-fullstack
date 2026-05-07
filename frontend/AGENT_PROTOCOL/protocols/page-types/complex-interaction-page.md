# 复杂交互页面协议

本协议约束 agent、chat、workbench、editor、builder、canvas、flow、console 等复杂交互页面。

这类页面最容易被 AI 写成一个巨大 panel、一个巨大 thread 或一个巨大 page hook。协议目标是让复杂业务天然分层，而不是事后人工重构。

---

## 1. 适用范围

出现以下任一信号，应按复杂交互页面处理：

- 同一页面包含多个可交互 panel。
- 有消息流、事件流、任务流、节点流或 activity stream。
- 页面需要把服务端事件转换成展示模型。
- 页面同时包含列表、详情、编辑器、抽屉、工具栏、状态区。
- `use-<page>-page.ts` 开始维护多个状态域。
- `components/*.tsx` 开始同时包含 JSX、事件映射、业务计算和大量 copy。

如果不确定，按本协议处理。复杂页面提前拆分的成本低于事后拆分巨大文件。

---

## 2. 标准目录

```txt
<page>/
  index.tsx
  types.ts
  constants.ts
  use-<page>-page.ts
  <page>-data.ts
  domain/
    logic.ts
  view-model/
    <panel>-view-model.ts
    <event>-handlers.ts
  components/
    page-header.tsx
    <panel>.tsx
    <thread>.tsx
    <detail-panel>.tsx
```

允许按真实业务继续拆 `domain/`、`view-model/`、`components/`，但不要发明含义模糊的 `utils/`、`helpers/`、`misc/` 来收容复杂度。

---

## 3. 文件职责

### `index.tsx`

只负责页面装配：

- 调用一个页面级 hook。
- 连接页面主区块。
- 传递少量 props。

不放 API、数据转换、复杂 `useMemo`、表单字段、消息列表渲染、panel 内部 JSX。

### `use-<page>-page.ts`

只负责编排：

- 调用 `<page>-data.ts` 中的数据入口。
- 维护页面级轻量状态，例如 selected id、open panel、active tab。
- 调用 `view-model/` 和 `domain/` 的纯函数。
- 暴露给 `index.tsx` 和 components 的 props/handlers。

不放：

- 大段 JSX。
- API import。
- reducer 实现。
- 大型事件归并、树转换、消息格式化。
- 大型常量或文案。

如果 hook 超过约 220 行，默认应拆出 `view-model/` 或 `domain/`。

### `<page>-data.ts`

只负责数据入口：

- query/mutation。
- API adapter 调用。
- mock/demo data 的入口。
- API response 到页面原始数据的轻量转换。

不放 JSX、panel view model、reducer、组件 props 拼装。

### `domain/`

只放纯业务逻辑：

- event/message/task 的排序、归并、分组。
- 状态机或 reducer 的纯实现。
- 权限、依赖、图结构、树结构计算。
- 可单测的规则。

不 import React，不写 JSX，不消费 i18n hook，不访问 API。

### `view-model/`

只放展示模型：

- domain/data 到 panel props 的转换。
- event 到 timeline/thread item 的转换。
- toolbar/action 状态派生。
- detail drawer、side panel、empty/error 状态模型。

不写 JSX，不请求 API，不沉淀共享 UI 协议。

### `components/`

只放页面局部 presentational components：

- 接收已经整理好的 props。
- 渲染局部 UI。
- 只保留与 DOM 事件直接相关的轻量 handler。

不放 API、React Query、reducer、复杂 `useMemo`、业务算法、view model 映射。若组件文件超过约 220 行，应优先拆成更小的局部组件或把逻辑移到 `view-model/` / `domain/`。

---

## 4. 禁止形态

以下形态默认违反本协议：

- `components/<page>-panel.tsx` 同时包含 mock data、事件转换、状态 reducer 和 JSX。
- `components/<page>-thread.tsx` 同时负责消息排序、消息分组、消息渲染和输入提交。
- `use-<page>-page.ts` 同时包含 API 调用、复杂 reducer、view model 拼装和所有事件 handler。
- 页面局部复制 DataTable、tabs、dialog、toolbar、pagination、toast、shell 等通用模式。
- 为了“灵活”在页面本地暴露 `className`、slot、render prop，然后复制共享层能力。

出现这些信号时，先拆分或上移能力，再继续写业务。

---

## 5. 共享能力判断

复杂页面仍然不是 app 私造组件库的例外。

默认顺序：

1. `@workspace/app-kit`
2. `@workspace/ui-components`
3. `@workspace/services`
4. 页面本地业务装配

若第二个页面也会复用某个 panel、toolbar、timeline、filter header、dialog、empty/error pattern，应重新判断是否进入 `@workspace/app-kit`。页面本地只保留当前业务私有组合。

---

## 6. 完成标准

复杂交互页面完成时必须满足：

- `index.tsx` 能一眼看出页面结构。
- `use-<page>-page.ts` 是编排层，不是逻辑仓库。
- API/data、domain logic、view model、presentational components 各有落点。
- 单个页面模块没有膨胀成几百行的不可治理文件。
- 用户可见文案通过 locale key 管理，runtime source 不硬编码 CJK 文案。
- 后续 AI 修改某个 panel、事件映射或业务逻辑时，能根据目录名找到正确文件。
