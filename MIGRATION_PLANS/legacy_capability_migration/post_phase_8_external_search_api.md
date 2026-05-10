# Post 8-9：Rust 外部搜索 Adapter API

状态：done

## 本次目标

- 把旧产品外部搜索接口的能力思想迁到新产品 Rust 后端。
- 新增 App 侧外部搜索 adapter 状态和查询接口。
- adapter 未配置时返回标准降级，不伪造实时来源。
- adapter 配置后才请求真实 provider，并统一归一化来源、时效、置信度和 provenance。

## 涉及范围

- 产品内核：外部搜索从 Agent deferred 契约推进到可调用的后端 service/web API。
- 后端：新增 `service::api::external_search`、`service::dto::external_search`、web-server handler/routes/statics。
- 配置：`backend/config/services-example.toml` 新增 `[external_search]` 示例，默认关闭。
- API：新增 `/api/app/external-search/status` 和 `/api/app/external-search/search`。
- 前端：本阶段不改前端。

## 用户可感知变化

当前用户登录后，后端可以真实返回外部搜索 adapter 是否配置。未配置时，接口返回 `degraded`、空来源和明确原因，不会假装已经查到外部数据。配置真实 provider 后，同一接口可以返回 `live`、来源列表、域名统计、发布时间统计和置信度。

## 后续缺口

- Agent `search_*` skill 还没有直接调用该 adapter。
- 前端还没有展示 `/api/app/external-search/*` 的来源、置信度和降级原因。
- 需要运维在部署环境配置真实 `[external_search]` provider。

## 当前验证

- [x] `cargo fmt`
- [x] `cargo test -p service external_search`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `git diff --check`
