# Web Server 层开发协议 (AI 专用)

> 本文档是 **执行规约**，AI 必须严格按步骤在 `crates/web-server` 中添加或修改 HTTP 接口。
> `web-server` 负责 HTTP 请求响应、参数校验、身份认证，以及同步维护 `../API_DOCS/` 下的 Markdown 接口文档。它直接调用 `service` 层的业务 API。

---

## 一、核心原则

1. **绝对禁止数据库操作**：此层绝不允许出现 `sea-orm` 或任何数据库特有类型（比如 `DbErr`、`Model`）。所有业务逻辑都应代理给 `service` 层的 API。
2. **Markdown 文档同步**：所有新增或修改的接口，都必须同步更新 `../API_DOCS/` 中对应的 Markdown 文档，保证文档和真实路由、请求结构、响应结构一致。
3. **分层错误处理**：业务层的 `Error`（如 NotFound, ValidationError）在这一层需要被转换为合适的 HTTP 状态码并封装成标准响应。
4. **强类型约束**：请求参数需通过 `validator` 和 `serde` 在进入 Handler 前完成校验。
5. **统一认证来源**：如果接口需要登录态，请按 `AUTH_INTEGRATION_GUIDE.md` 集成 `auth` 服务，不要在当前业务服务内重复实现登录或 JWT 签发。
6. **先确认接口设计**：如果“服务端开发文档”还没有被用户确认，不要直接开始写 Handler 和路由。
7. **Agent 统一入口**：如果接口需要调用大模型，只能通过 `agent` crate 和 `statics::agent_manager` 运行 agent session，不要在 handler 中直接创建模型 client。

---

## 二、标准开发流程

当需要新增一个业务模块（例如 `user_biz`）的 HTTP 接口时，请遵循以下步骤：

### Step 1: 编写 Web DTO (`dto/xxx.rs`)

在 `crates/web-server/src/dto/user_biz.rs` 中定义 HTTP 专用的请求和响应结构，并加上 `validator` 注解：

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 创建用户请求参数
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 20))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
}

/// HTTP 通用响应数据体
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
}
```

### Step 2: 编写 Handler (`handlers/xxx.rs`)

在 `crates/web-server/src/handlers/user_biz.rs` 中编写处理函数。通过依赖注入（如 `get_default_ctx` 或提取器）获取所需的上下文，调用 `service` 层，然后返回 `toolcraft_axum_kit::ResponseResult`。

```rust
use axum::{extract::Query, Json};
use toolcraft_axum_kit::{CommonResponse, IntoCommonResponse, ResponseResult};
// 引入下层业务服务
use service::api::user_biz::UserBizApi;
// 引入全局静态依赖
use crate::{dto::user_biz::*, error::Error, statics::db_manager::get_default_ctx};

pub async fn create_user_handler(
    Query(req): Query<CreateUserRequest>,
) -> ResponseResult<UserResponse> {
    // 1. 初始化 service 层的业务 API
    let api = UserBizApi::new(get_default_ctx());

    // 2. 将 Web 层 DTO 转为 Service 层 DTO 并调用业务层
    let biz_req = service::dto::user_biz::CreateUserRequest {
        username: req.username,
        email: req.email,
    };
    
    // 3. 调用业务逻辑并进行错误转换
    let biz_resp = api.create_user(biz_req)
        .await
        .map_err(|e| Error::Custom(e.to_string()))?; // 注意: 实际项目中建议将错误映射为适当的 HTTP Error

    // 4. 将业务结果映射为 HTTP 层响应
    let resp = UserResponse {
        id: biz_resp.id,
        username: biz_resp.username,
        email: biz_resp.email,
    };

    Ok(resp.into_common_response().to_json())
}
```

如果接口需要登录态，应额外提取：

```rust
use axum::Extension;
use toolcraft_axum_kit::middleware::auth_mw::AuthUser;

pub async fn my_handler(
    Extension(auth_user): Extension<AuthUser>,
) {
    let user_id = auth_user.user_id;
}
```

如果接口需要 AI/LLM 调用，应运行 agent session：

```rust
use crate::statics::agent_manager::run_agent_stream;

let events = run_agent_stream(
    auth_user.user_id,
    req.workspace_id,
    req.agent_id,
    req.session_id,
    req.input,
)?;
```

LLM 配置和调用规则见 `AI_PROTOCOLS/LLM_CLIENT_GUIDE.md`。

如果接口是 agent 流式输出，应使用 Axum SSE，并遵守：

- 只在 agent session stream 场景使用 SSE
- WebSocket 继续只做通知和业务推送
- SSE endpoint 优先使用 `POST + fetch stream`
- SSE 事件来自 `agent::api::SessionEvent::event_name()`

### Step 3: 注册 Routes (`routes/xxx.rs`)

在 `crates/web-server/src/routes/user_biz.rs` 中定义子路由结构。如果接口需要登录态，请给该子路由挂 `auth::<VerifyJwt>` 中间件：

```rust
use axum::{Router, middleware::from_fn, routing::post};
use toolcraft_axum_kit::middleware::auth_mw::auth;
use toolcraft_jwt::VerifyJwt;

use crate::handlers::user_biz::create_user_handler;

pub fn user_biz_routes() -> Router {
    Router::new()
        .route("/users", post(create_user_handler))
        .route_layer(from_fn(auth::<VerifyJwt>))
}
```

### Step 4: 挂载到主路由 (`routes/mod.rs`)

在 `crates/web-server/src/routes/mod.rs` 中将子路由挂载上去。

```rust
mod user_biz; // 添加这个模块

use std::sync::Arc;
use axum::{Extension, Router};
use toolcraft_axum_kit::middleware::cors::create_cors;
use toolcraft_jwt::VerifyJwt;

pub fn create_routes(jwt: Arc<VerifyJwt>) -> Router {
    let cors = create_cors();

    Router::new()
        .nest("/api/user-biz", user_biz::user_biz_routes())
        .layer(Extension(jwt))
        .layer(cors)
}
```

### Step 5: 导出模块

别忘了在 `crates/web-server/src/dto/mod.rs` 和 `crates/web-server/src/handlers/mod.rs` 中声明对应的新模块：
```rust
pub mod user_biz;
```

### Step 6: 更新 Markdown API 文档

在 `../API_DOCS/` 中补充或更新对应模块的 Markdown 文档，至少包含：

- 实际路由前缀
- 请求方法和路径
- 请求体或路径参数
- 字段约束
- 成功响应示例
- 关键权限要求

---

## 三、自检清单

- [ ] 在开始写 web-server 代码前，接口设计是否已经被用户确认？
- [ ] 是否确保此层没有引入 `sea-orm` 或调用任何原 `repo` 层的直接方法？
- [ ] `../API_DOCS/` 中对应的 Markdown 文档是否已经同步更新？
- [ ] `routes/mod.rs` 中是否已经正确挂载实际路由前缀？
- [ ] 数据结构的参数验证是否完整使用了 `validator` 标签？
- [ ] 如果接口调用 AI/LLM，是否通过 `statics::agent_manager`，并同步更新了配置文档和验证命令？
