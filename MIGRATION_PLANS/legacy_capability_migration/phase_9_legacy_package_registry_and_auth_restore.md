# 第 9 项：登录态恢复与旧产品内置能力包治理同步

状态：done

## 本次目标

- 修复登录后刷新或直接访问受保护页面误跳 `/login` 的问题。
- 把旧产品 `server/packages` 中的内置 domain、role、skill manifest 同步到新产品能力包治理目录。
- 保持边界清楚：本次同步的是治理目录和能力矩阵元数据，不等于 Agent runtime 已动态执行旧 Python skill。

## 产品内核变化

- 能力包治理从“空目录或手工录入”升级为“可一键同步旧产品内置能力生态”。
- 旧产品 manifest 中的 4 个 domain、6 个 role、10 个 skill 包进入当前用户自己的治理目录。
- 从旧 manifest 的能力字段派生出 32 个 capability 记录，用于后续能力矩阵、校验、运行锁和 runtime adapter 开发。
- role manifest 里的旧 skill 函数名不再作为新治理依赖校验，避免误报缺依赖；这些旧函数名保存在 metadata 的 `legacySkills` 中，后续开发 adapter 时可追溯。

## 后端变化

- 新增 `POST /api/app/packages/sync-legacy-builtins`。
- 新增 `PackageGovernanceApi::sync_legacy_builtin_packages`。
- 复用 `package_registry` 和 `package_state_history`，不新增表。
- 批量同步使用事务，避免中途失败留下半同步状态。
- 默认更新已有包的名称、版本、manifest 路径、依赖、兼容性和 metadata，但保留用户当前状态；只有 `resetStatus=true` 才重置状态。
- 同步 `API_DOCS/app.md`。

## 前端变化

- `App.tsx` 修复登录态恢复时序：本地有 token 时，等待 `/me` 和 App 权限恢复写入 Redux 后再创建受保护路由。
- 能力包治理页增加“同步旧产品能力包”按钮。
- 前端通过 `syncLegacyBuiltinPackagesApi` 调用真实后端接口，成功后刷新能力包目录。
- 中英文文案已补齐。

## 用户可感知变化

用户登录后直接打开或刷新 `/packages`、`/products` 等受保护页面，不会再因为 Redux 尚未恢复而误跳到登录页。

用户进入“能力包治理”页面后，可以点击“同步旧产品能力包”。同步后页面能看到旧产品内置的 domain、role、skill 和 capability 治理记录，例如 `Builtin Ops Skill Pack`。这些记录可继续搜索、筛选、校验、启用、停用、回滚和保存运行锁。

## 已验证

- `cargo fmt`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo check`
- `cargo clippy`
- `pnpm -C frontend/apps/web rules`
- `pnpm -C frontend/apps/web lint`
- `pnpm -C frontend/apps/web test`
- `pnpm -C frontend/apps/web build`
- 真实 HTTP 登录 `test123456/test123456` 后调用同步接口：
  - 同步前目录总数：0
  - 首次同步创建：52
  - manifest 直接记录：20
  - 派生 capability：32
  - 同步后目录总数：52
  - `skill/builtin.ops/validate` 通过
  - `role/role.ops-strategist/validate` 通过
- 浏览器自动化验证：
  - 登录后直达 `/packages` 不跳登录
  - 刷新 `/packages` 不跳登录
  - 直达 `/products` 不跳登录
  - 刷新 `/products` 不跳登录
  - 能力包页可见“同步旧产品能力包”和 `Builtin Ops Skill Pack`
- LLM / Agent 验证：
  - `http://119.29.232.53:16814/v1/models` 可访问，包含 `gpt-oss:20b` 和 `gemma4:26b`
  - `gpt-oss:20b` 直接 `chat/completions` 返回 `LLM_OK`
  - `cargo run -p agent --bin ollama_task_graph_smoke -- --config config/services.toml --llm ollama-gpt-oss-20b ...` 跑通，出现 `started -> route_decided -> output -> finished`
  - 真实 `POST /api/sse/agent/session/stream` 协同任务跑通，返回 61 个 SSE 事件，包含 `task_graph_planned`、`output`、`finished`，未出现 `LLM request failed`

## 未完成边界

- 旧产品核心 skill 的 Rust runtime adapter 尚未全部迁移。
- 外部搜索、平台真实执行、教学、导出、素材生成仍需后续能力项继续开发。
- 云端 Ollama 与当前默认 `gpt-oss:20b` 已验证可用；后续仍需继续压测更复杂任务、长输出、超时和 `gemma4:26b` 内容为空的问题。
