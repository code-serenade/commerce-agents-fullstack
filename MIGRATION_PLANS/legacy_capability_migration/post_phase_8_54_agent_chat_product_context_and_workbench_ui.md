# Post 8-54：问答商品上下文显式选择与工作台遮挡修正

状态：done

## 目标

本轮修正问答主链路中最影响演示和真实使用的几个问题：

- 未选择预设商品时，不允许后端隐式注入任何商品上下文。
- 预设商品入口必须清楚显示，且能包含预售/上架中商品。
- 左上角折叠按钮不能重复。
- 右侧虚拟员工工作区不能遮挡正文。
- 过程卡片必须在正文回答之前出现，回答结束后默认折叠。

## 后端变化

- `BuildAgentRetrievalContextRequest` 新增 `allow_implicit_products`。
- Agent SSE 问答链路传入 `allow_implicit_products=false`，不再用用户问题里的“咖啡”等关键词自动检索已有商品。
- `GetWorkspaceContextSummaryRequest` 和 `GetWorkspaceContextPackRequest` 新增 `include_workspace_product_context`。
- 普通工作区 API 继续允许使用工作区绑定商品；Agent SSE 问答链路禁用该回退，必须由本轮 `productIds/contextRefs` 显式选择商品。
- `API_DOCS/sse.md` 已明确：未选择商品时，不注入默认商品、商品知识、商品资产或商品素材。

## 前端变化

- Agent Chat shell 关闭 TopBar 里的重复 sidebar trigger，只保留 `SidebarShell` 自己的折叠机制。
- 任务设置摘要新增“未指定商品 / 已选商品”入口，点击即可展开商品选择。
- 商品列表加载从 `active` 扩展到 `active/listing/draft`，覆盖运营中、上架中/预售、草稿商品。
- 右侧虚拟员工工作区从覆盖式浮层改为真实分栏或窄屏下方区域；收起按钮也在布局内，不再压住正文。
- 过程卡片挂在最新用户消息之后、助手正文之前。
- 过程卡片运行时展开，结束后默认折叠，用户仍可手动展开查看执行细节。

## 用户可感知变化

- 用户不勾选商品时，界面明确显示“未指定商品”，后端按普通电商问题回答。
- 用户勾选某个预设商品后，本轮回答才会读取该商品资料。
- 问题里出现“咖啡”不会自动把保存过的咖啡商品带入回答。
- 过程信息不会再挤在答案后面造成“好像还没结束”的误解。
- 右侧虚拟员工区域不再遮挡主回答。
- 左上角只保留一个清晰的折叠入口。

## 验证

- [x] `cargo fmt`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `cargo test -p service agent_context`
- [x] `cargo test -p service workspace::tests::context`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`

## 说明

本轮没有新增假能力，也没有把旧产品代码直接搬进新产品。商品上下文仍通过新产品既有 SSE metadata、service API、workspace context 和前端 app 页面装配体系实现。

Codex 内置浏览器插件本轮因 Windows 会话凭据错误无法截图接管；已用 Edge 打开 `http://localhost:5173/commerce` 供人工试用。
