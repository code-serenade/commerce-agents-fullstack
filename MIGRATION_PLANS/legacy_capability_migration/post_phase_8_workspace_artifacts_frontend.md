# Post Phase 8：工作区交付物刷新可见

状态：done

基础 8 个迁移项已经完成。本次属于第 1 个体验闭环补强项，目标是让 Phase 4 已经沉淀到后端的 workspace artifact，在完整工作区页面刷新后仍然可查看。

## 目标

- 工作区页面读取真实后端 artifact API。
- 用户进入执行工作区后，可以在选中工作区下查看交付物列表。
- 用户选择交付物后，可以查看后端保存的正文或预览，以及格式、存储、来源、运行 ID、校验状态。
- 前端不伪造交付物，只消费 `GET /api/app/workspaces/{workspaceId}/artifacts` 和 `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}`。

## 产品内核变化

- 问答页生成的最终回答不再只停留在 SSE 过程面板。
- `workspace_artifact_created` 事件背后的后端持久化结果，现在能在业务工作区页面被再次读取。
- 产品从“一次问答过程可见”补强为“问答交付物可沉淀、可刷新、可复用”。

## 后端变化

- 本次未新增后端接口。
- 复用已有 workspace artifact 后端能力：
  - `GET /api/app/workspaces/{workspaceId}/artifacts`
  - `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}`
- 复用已有鉴权、用户隔离、工作区归属校验和 active 状态过滤。

## 前端变化

- `frontend/apps/web/src/api/commerce.ts`
  - 新增 `WorkspaceArtifactResponse`、`WorkspaceArtifactListResponse`。
  - 新增 `listWorkspaceArtifactsApi`、`getWorkspaceArtifactApi`。
- `frontend/apps/web/src/pages/site/commerce/workspaces/use-workspace-queries.ts`
  - 工作区选中后自动拉取 artifact 列表。
  - 自动选中第一个 artifact，支持点击列表切换详情。
- `frontend/apps/web/src/pages/site/commerce/workspaces/components/workspace-detail-panel.tsx`
  - 新增“交付物”tab。
  - 新增交付物列表和详情预览。
- `frontend/apps/web/src/pages/site/commerce/workspaces/components/workspace-table-columns.tsx`
  - 新增 artifact 列定义。
- `frontend/apps/web/src/pages/site/commerce/workspaces/workspace-metrics.ts`
  - 工作区指标新增交付物数量和已验收数量。
- `frontend/apps/web/lang/zhCN/messages.ts`
- `frontend/apps/web/lang/en/messages.ts`
  - 新增交付物相关中英文文案。

## 用户可感知变化

- 用户打开“执行工作区”，选中某个工作区后，会在详情区域看到“交付物”tab。
- 点击“交付物”tab 后，左侧列表显示后端保存的报告、文件或阶段交付结果。
- 点击某个交付物后，右侧详情区显示标题、类型、来源、校验状态、格式、存储方式、文件信息、运行 ID 和正文/预览。
- 如果刚才在问答页完成了一次 Agent 任务并生成 final report，刷新工作区页面后仍能在这里看到对应交付物。

## 验收

- [x] 前端 API client 接入真实后端 artifact 接口。
- [x] 工作区页面刷新后可拉取 artifact 列表。
- [x] 工作区页面可查看 artifact 详情。
- [x] 前端文案已补齐。
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`
- [x] 使用测试账号 `test123456` 登录后完成真实 HTTP smoke：
  - 登录成功。
  - 创建临时工作区成功。
  - 创建临时 `final_report` artifact 成功。
  - artifact 列表查询返回 `total=1`。
  - artifact 详情查询返回正文和 `validationStatus=unchecked`。
  - 临时工作区已归档清理。

## 运行注意

本次 smoke 前，当前本地 `19878` 上运行的旧 `web-server.exe` 对 artifact 路由返回 404。重启后端并加载当前代码后，artifact 路由恢复正常。
