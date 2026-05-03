# Repo 层开发协议 (AI 专用)

> 本文档是 **执行规约**，AI 必须严格按步骤在 `crates/repo` 中添加或修改单表逻辑。
> `repo` 也就是之前的 `pg-tables` 层，职责为：表级事实模型 + 单表操作。

---

## 一、核心原则

1. **单表原则**：`repo` 中的 `service` 方法**只能操作一张表**，绝对不能跨表 JOIN。
2. **避免业务逻辑**：不要在 `repo` 层的 `service` 中验证外键（比如检查 User 是否存在）。这是上层 `service`（原 demo-db）的职责。
3. **数据转换**：负责 Entity Model 和 领域 DTO 之间的互相转换。
4. **生成而非手写**：`entity` 目录下的文件全部由 `sea-orm-cli` 自动生成，**禁止手写或修改 entity 代码**。
5. **不要误解认证用户 ID**：如果字段来自 JWT 中的当前用户 `user_id`，默认按字符串处理，不要擅自假设是 `i64`。
6. **事务协作**：repo 方法必须可被 `service` 层事务编排复用，不在 repo 内开启/提交/回滚事务。

---

## 二、标准开发流程

当需要为一个新表（假设表名叫 `user_profile`）添加 `repo` 支持时，遵循以下步骤：

### Step 1: 确保 Entity 已生成

在数据库已经存在最新表结构的前提下，运行 `./scripts/generate_entity.sh` 或 `make entity-generate`，确保 `crates/repo/src/entity/user_profile.rs` 被生成，并在 `crates/repo/src/entity/mod.rs` 中被正确导出。

不要为了生成 entity 默认执行 `fresh_db.sh`，因为它会重建数据库。

### Step 2: 创建模块目录

在 `crates/repo/src/table/` 下创建新表对应的目录：

```bash
mkdir -p crates/repo/src/table/user_profile
```

### Step 3: 定义 DTO (`dto.rs`)

在 `crates/repo/src/table/user_profile/dto.rs` 中定义数据传输对象：

```rust
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// 事实结构体（完整数据，对应数据库表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i64,
    pub user_id: String,
    pub bio: String,
    pub created_at: OffsetDateTime,
}

/// 创建参数（不含 id、created_at，这些由 Service / 数据库处理）
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserProfile {
    pub user_id: String,
    pub bio: String,
}

/// 查询/更新参数
#[derive(Debug, Clone, Deserialize)]
pub struct QueryUserProfile {
    pub user_id: Option<String>,
}
```

### Step 4: 实现单表 Service (`service.rs`)

在 `crates/repo/src/table/user_profile/service.rs` 中使用 `db_core::impl_repository!` 宏并实现核心方法：

```rust
use sea_orm::*;
use db_core::{DbContext, error::BizResult};

// 引入自动生成的 entity
use crate::entity::user_profile;
// 引入 DTO
use super::dto::{UserProfile, CreateUserProfile};

// 1. 使用宏生成基础 Repository
db_core::impl_repository!(UserProfileRepo, user_profile::Entity, user_profile::Model);

// 2. 定义 Service 结构体
pub struct UserProfileService {
    db: DbContext,
}

impl UserProfileService {
    pub fn new(db: DbContext) -> Self {
        Self { db }
    }

    // 3. 实现创建逻辑 (不验证 user_id 对应的 User 是否存在)
    pub async fn create(&self, input: CreateUserProfile) -> BizResult<UserProfile> {
        let model = user_profile::ActiveModel {
            user_id: Set(input.user_id),
            bio: Set(input.bio),
            created_at: Set(time::OffsetDateTime::now_utc()),
            ..Default::default()
        };
        
        let result = UserProfileRepo::insert(&self.db, model).await?;
        Ok(Self::from_model(result))
    }

    // 4. 实现查询逻辑
    pub async fn get(&self, id: i64) -> BizResult<UserProfile> {
        let model = UserProfileRepo::find_by_id(&self.db, id)
            .await?
            .ok_or_else(|| db_core::error::Error::Custom(format!("UserProfile not found: {}", id)))?;
            
        Ok(Self::from_model(model))
    }
    
    // 按条件查询列表
    pub async fn list_by_user(&self, user_id: &str) -> BizResult<Vec<UserProfile>> {
        let models = UserProfileRepo::find(&self.db)
            .filter(user_profile::Column::UserId.eq(user_id))
            .all(&self.db.db)
            .await?;
            
        Ok(models.into_iter().map(Self::from_model).collect())
    }

    // 5. 内部模型转换方法
    fn from_model(model: user_profile::Model) -> UserProfile {
        UserProfile {
            id: model.id,
            user_id: model.user_id,
            bio: model.bio,
            created_at: model.created_at,
        }
    }
}
```

### Step 5: 导出模块

1. 在 `crates/repo/src/table/user_profile/mod.rs` 中导出：

```rust
pub mod dto;
pub mod service;

pub use dto::*;
pub use service::*;
```

2. 在 `crates/repo/src/table/mod.rs` 中注册新模块：

```rust
pub mod user_profile;
```

---

## 三、自检清单

- [ ] 新建的 Service 是否**只操作了当前表**？
- [ ] DTO 中的字段名称和类型是否和 Entity（即数据库结构）**严格一致**？
- [ ] 是否将业务判断（比如外键关系、权限校验）留给了上层？
- [ ] 创建资源时，`id` 和 `created_at` 等自增/时间戳字段是否由 Service 或数据库默认值生成，而不是由外部传入？
- [ ] 是否没有在 repo 层调用 begin/commit/rollback？
- [ ] 方法签名是否支持由 `service` 传入上下文（便于事务内复用）？
