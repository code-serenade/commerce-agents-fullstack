# Post 8-41：Design 交付物契约补强

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 41 个补强项：`Post 8-41`。

本次目标：

- 把设计岗位 native skill 的输出补齐为新产品统一成功契约。
- 让已有设计类 workspace artifact 映射稳定生效。
- 保持边界清楚：本阶段生成设计 brief/spec，不生成真实图片文件。

涉及范围：

- 产品内核：设计岗位从“当轮设计方案”推进到“可复用的设计交付物”。
- 后端 agent：`design_*` skill 输出补齐 `status=success`、`has_data`、`adapter_status`、`data_readiness` 和 `summary`。
- 后端 service：补充 `design_material_spec` 物化测试，继续沿用已有 `WorkspaceApi::materialize_agent_content_artifact`。
- API/SSE 文档：同步 design artifact 类型、默认规则披露和不生成真实图片文件的语义。

用户可感知变化：

- 用户让设计岗位做主图、详情页、配色、素材规范或活动海报方案时，成功结果可以在当前 workspace 真实新增对应设计交付物。
- 这些交付物沿用现有 artifact 列表、详情、预览和导出链路。
- 如果使用默认商品或平台规则，结果会通过 `data_readiness=calculated_from_defaults` 明确告知前端和用户。
- 本阶段不输出图片文件，前端不能展示为“图片已生成”。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent design_ -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_design_ -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`
