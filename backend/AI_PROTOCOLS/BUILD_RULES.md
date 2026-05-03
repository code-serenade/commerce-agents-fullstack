# 构建与脚本规则

## 基本规则

在本仓库执行 build、check、lint、test、run、migration 和 codegen 操作时：

- 这是一个 Rust workspace，优先使用 `cargo` 命令，不要使用前端包管理器工作流。
- 除非用户明确要求不要格式化，否则在其它 Rust 构建或验证命令前先运行 `cargo fmt`。
- 优先使用仓库根目录 workspace 命令，例如 `cargo check`、`cargo clippy` 和 `cargo test`。
- 如果只验证某个 crate，优先使用 `cargo <cmd> -p <crate>`。

## 完成前必须处理

除非用户明确要求不运行，或当前环境阻止执行，否则 AI 在认为代码修改完成前必须处理以下命令：

```bash
cargo fmt
cargo check
cargo clippy
```

- 如果修改了 API，完成前必须同步更新 `../API_DOCS/` 下的 Markdown API 文档。
