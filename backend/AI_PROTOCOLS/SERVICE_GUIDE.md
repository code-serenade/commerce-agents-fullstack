# Service 层开发协议 (AI 专用)

> 本文档是 **执行规约**，AI 必须严格按步骤在 `crates/service` 中添加或修改综合业务逻辑。
> `service` 也就是之前的 `demo-db` 层，职责为：业务 API 层，编排多个 Repo，处理业务语义，不直接操作数据库。

---

## 一、核心原则

1. **跨表编排**：`service` 层的 API 是唯一允许进行多表数据组合的地方。
2. **禁止直接访问数据库**：所有对数据的读写**必须**通过调用 `repo`（原 `pg-tables`）层中提供的单表 Service 进行。绝不能直接使用 `sea-orm` 的查询构建器直接操作数据库。
3. **业务逻辑中心**：外键校验、权限判断、数据聚合、复杂的计算逻辑都应该放在这一层。
4. **面向业务的 DTO**：返回给上层（Web HTTP 层）的 DTO 应该是具有业务语义的复合结构，而非单一的表结构。
5. **设计先行**：如果用户还没有确认“服务端开发文档”，不要直接开始实现业务 API。
6. **登录用户 ID 默认是字符串**：如果参数来自 JWT 的当前用户 `user_id`，默认按 `String` 处理，不要擅自定义成 `i64`。
7. **写操作优先考虑事务**：凡是“多次写入必须一起成功或一起失败”的业务，必须在 `service` 层使用事务。

---

## 二、事务规则（AI 必须执行）

### 1) 什么时候必须使用事务

满足任一条件就必须使用事务：

1. 一个 API 内有 **2 次及以上写操作**（insert/update/delete），并且业务要求原子性。
2. 涉及多表写入，任一表失败会导致数据不一致。
3. 需要先写 A 再写 B，且 B 失败时必须撤销 A。
4. 有库存/余额/配额/状态机流转等“不能部分成功”的场景。

### 2) 什么时候可以不使用事务

仅在以下情况可不使用事务：

1. 纯读操作（只查不写）。
2. 单次写操作，且允许失败后重试，不会造成业务不一致。
3. 明确采用最终一致性方案（异步补偿），并在文档中说明。

### 3) AI 默认策略

对“写接口”默认按**需要事务**处理，除非用户明确说明该接口允许部分成功。

### 4) 事务边界要求

1. 事务只能在 `service` 层开启。
2. `repo` 层不负责开启/提交/回滚事务，只负责单表能力。
3. 同一业务事务中的所有 repo 调用必须共享同一个事务上下文。

---

## 三、标准开发流程

假设我们要实现一个获取用户及其资料概览的业务功能，涉及 `user` 和 `user_profile` 两张表。

### Step 1: 定义业务 DTO (`dto/xxx.rs`)

在 `crates/service/src/dto/user_biz.rs` 中定义业务所需的请求和响应结构。

```rust
use serde::{Deserialize, Serialize};
// 引入 repo 层的单表 DTO 进行复用（如果需要）
use repo::table::user::User;
use repo::table::user_profile::UserProfile;

/// 业务请求参数
#[derive(Debug, Clone, Deserialize)]
pub struct GetUserOverviewRequest {
    pub user_id: String,
}

/// 业务复合响应（聚合了多表数据）
#[derive(Debug, Clone, Serialize)]
pub struct UserOverviewResponse {
    pub user: User,
    pub profiles: Vec<UserProfile>,
    // 还可以包含业务计算得出的字段
    pub profile_count: usize,
}
```

### Step 2: 实现业务 API (`api/xxx.rs`)

在 `crates/service/src/api/user_biz.rs` 中创建 API 结构体，持有对应的 Repo Service，并实现业务方法。

```rust
use db_core::{DbContext, error::BizResult};
use repo::table::user::UserService;
use repo::table::user_profile::UserProfileService;

use crate::dto::user_biz::{GetUserOverviewRequest, UserOverviewResponse};

pub struct UserBizApi {
    user_svc: UserService,
    profile_svc: UserProfileService,
}

impl UserBizApi {
    // 1. 初始化时注入依赖的 Repo Service
    pub fn new(db: DbContext) -> Self {
        Self {
            user_svc: UserService::new(db.clone()),
            profile_svc: UserProfileService::new(db),
        }
    }

    // 2. 编写业务逻辑方法
    pub async fn get_user_overview(&self, req: GetUserOverviewRequest) -> BizResult<UserOverviewResponse> {
        // 编排 1: 获取主表数据。如果不存在，底层 repo svc 会返回 not_found 错误
        let user = self.user_svc.get(&req.user_id).await?;

        // 编排 2: 获取关联表数据
        let profiles = self.profile_svc.list_by_user(&req.user_id).await?;
        
        // 业务计算
        let profile_count = profiles.len();

        // 3. 组合响应
        Ok(UserOverviewResponse { 
            user, 
            profiles,
            profile_count,
        })
    }
}
```

### Step 2.1: 写操作使用事务（必须）

当业务涉及多次写操作时，必须在 `service` 层用事务包裹。

以下为**协议示意代码**（具体参数类型按你当前 `db-core-rs` 版本 API 为准）：

```rust
pub async fn create_xxx(&self, req: CreateXxxReq) -> BizResult<CreateXxxResp> {
    self.db
        .transaction(|tx_ctx| {
            Box::pin(async move {
                // 使用同一个事务上下文执行所有写操作
                // 可以是 repo 提供的事务友好方法，也可以是同一批 repo 调用
                self.repo_a.create_with_ctx(tx_ctx, req.a).await?;
                self.repo_b.create_with_ctx(tx_ctx, req.b).await?;
                Ok(())
            })
        })
        .await?;

    Ok(CreateXxxResp {})
}
```

说明：

1. `transaction` 回调返回 `Ok` 会提交，返回 `Err` 会回滚。
2. 同一事务里的所有写操作必须使用同一个 `tx_ctx`。
3. 不要在 `repo` 层写 begin/commit/rollback。

### Step 2.2: 使用 begin / commit / rollback（复杂场景）

当你遇到以下情况时，可使用手动事务：

1. 闭包生命周期难以处理，`transaction` 写法受限。
2. 需要更细粒度的事务边界控制（例如中间分段处理、条件化提交前校验）。
3. 需要显式控制嵌套事务或保存点逻辑。

示意代码：

```rust
pub async fn create_xxx_manual_tx(&self, req: CreateXxxReq) -> BizResult<CreateXxxResp> {
    let tx = self.db.begin().await?;

    // 使用同一个事务上下文进行所有写操作
    let repo_a = RepoAService::new(tx.clone());
    let repo_b = RepoBService::new(tx.clone());

    let result: BizResult<CreateXxxResp> = async {
        let a = repo_a.create(req.a).await?;
        repo_b.create_by_a(a.id, req.b).await?;
        Ok(CreateXxxResp { id: a.id })
    }
    .await;

    match result {
        Ok(resp) => {
            tx.commit().await?;
            Ok(resp)
        }
        Err(err) => {
            tx.rollback().await?;
            Err(err)
        }
    }
}
```

选择规则：

1. 默认优先 `transaction(...)`。
2. 只有在确实需要复杂控制流时，才使用 `begin/commit/rollback`。
3. 两种方式都必须在 `service` 层开启事务。

### Step 3: 导出模块

1. 在 `crates/service/src/dto/mod.rs` 中注册：
```rust
pub mod user_biz;
```

2. 在 `crates/service/src/api/mod.rs` 中注册：
```rust
pub mod user_biz;
```

3. 如果需要暴露给外部，也可以在 `crates/service/src/lib.rs` 中处理。

---

## 四、自检清单

- [ ] 是否在 API 中注入了 `repo` 层对应的 `XXXService`？
- [ ] 数据读写是否全部**通过 `repo` 的 Service 方法**调用完成？
- [ ] 是否避免了在 `service` 中引入 `sea_orm` 和数据库驱动的细节代码？
- [ ] 业务校验（如“如果用户状态为禁用则不允许获取数据”）是否都实现在了本层？
- [ ] 对写接口是否判断了“是否必须事务”？
- [ ] 若必须事务，是否在 `service` 使用了 `DbContext::transaction(...)`？
- [ ] 若使用手动事务，是否正确覆盖了 commit/rollback 分支？
- [ ] 事务内是否全部使用同一个 `tx` 上下文构建的 repo 实例？
