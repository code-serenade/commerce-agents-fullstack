# migration 使用说明

这个 crate 负责数据库迁移。

## 常用命令

### 生成新的迁移文件

```sh
cargo run -- generate MIGRATION_NAME
```

### 执行全部待处理迁移

```sh
cargo run
```

或：

```sh
cargo run -- up
```

### 执行前 10 个待处理迁移

```sh
cargo run -- up -n 10
```

### 回滚最近一次迁移

```sh
cargo run -- down
```

### 回滚最近 10 次迁移

```sh
cargo run -- down -n 10
```

### 删除全部表后重新执行所有迁移

```sh
cargo run -- fresh
```

### 回滚全部迁移后重新执行

```sh
cargo run -- refresh
```

### 回滚全部已执行迁移

```sh
cargo run -- reset
```

### 查看迁移状态

```sh
cargo run -- status
```
