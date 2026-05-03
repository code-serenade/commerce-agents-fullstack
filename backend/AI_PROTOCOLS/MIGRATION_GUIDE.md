# Migration 开发协议 (AI 专用)

> 本文档是 **执行规约**，AI 必须严格按步骤在 `crates/migration` 中添加或修改表。

---

## 一、核心原则

1. **Sea-ORM 优先**：必须使用 `sea-orm-migration` 提供的 DSL (Domain Specific Language) 定义表结构。
2. **幂等性**：使用 `.if_not_exists()` 创建表。
3. **回滚支持**：每个 `up` 方法必须对应一个 `down` 方法（通常是 `drop_table`）。
4. **命名规范**：
   - 表名和列名使用 `PascalCase` 定义在 `enum` 中。
   - 数据库实际生成的表名和列名遵循 Sea-ORM 的默认转换（通常是 `snake_case`）。
5. **依赖管理**：如果新表依赖于其他 migration 中的表，确保在 `m000x_xxx.rs` 中正确引用。
6. **先确认再落库**：如果用户还没有确认表结构，先停止编码，回到“服务端开发文档”阶段。

---

## 二、标准开发流程

### Step 1: 生成新 Migration 文件

使用以下命令生成带时间戳的新 migration 文件：

```bash
sea-orm-cli migrate generate <table_name> -d crates/migration
```

### Step 2: 定义 Iden Enum

在文件末尾定义 `enum` 来表示表名和列名。

```rust
#[derive(DeriveIden)]
enum User {
    Table,
    UserId,
    Username,
    Email,
    CreatedAt,
}
```

### Step 3: 实现 `up` 方法

使用 `manager.create_table` 定义结构：

```rust
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(User::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(User::UserId)
                        .big_integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(User::Username).string().not_null())
                .col(ColumnDef::new(User::Email).string().unique_key().not_null())
                .col(
                    ColumnDef::new(User::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .to_owned(),
        )
        .await
}
```

### Step 4: 实现 `down` 方法

通常使用 `drop_table`：

```rust
async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
        .drop_table(Table::drop().table(User::Table).cascade().to_owned())
        .await
}
```

### Step 5: 注册 Migration

在 `crates/migration/src/lib.rs` 中注册新的 migration 模块。

1. 添加 `mod m000x_xxx;`
2. 在 `MigratorTrait::migrations` 的 `vec![]` 中添加 `Box::new(m000x_xxx::Migration)`。

---

## 三、常用字段定义模版

| 类型 | DSL 定义 |
|------|----------|
| 自增主键 (i64) | `.big_integer().not_null().auto_increment().primary_key()` |
| 字符串 (VARCHAR) | `.string().not_null()` |
| 可空字符串 | `.string().null()` |
| JSONB | `.json_binary().not_null()` |
| 带时区的 Timestamp | `.timestamp_with_time_zone().not_null().default(Expr::current_timestamp())` |
| 外键 | `.foreign_key(ForeignKey::create().from(Self::Table, Self::OtherId).to(Other::Table, Other::Id).on_delete(ForeignKeyAction::Cascade))` |

---

## 四、外键引用规则

如果需要引用其他 migration 定义的表（例如 `m0001` 中定义的 `Metric`）：

1. 在当前 migration 文件顶部导入：`use crate::m0001_phase_a_core::Metric;`
2. 在 `foreign_key` 定义中使用 `Metric::Table` 和 `Metric::MetricId`。

---

## 五、自检清单

- [ ] 运行 `sea-orm-cli migrate generate` 生成文件了吗？
- [ ] 字段类型是否正确（特别是 `big_integer` 对应 Rust 的 `i64`）？
- [ ] 是否在 `lib.rs` 中注册了？
- [ ] `down` 方法是否包含 `.cascade()` 以确保清理干净？
- [ ] 是否使用了 `.if_not_exists()`？
