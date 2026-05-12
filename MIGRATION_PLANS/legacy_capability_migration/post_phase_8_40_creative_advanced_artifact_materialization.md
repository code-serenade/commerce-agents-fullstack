# Post 8-40：Creative 高级内容交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 40 个补强项：`Post 8-40`。

本次目标：

- 把旧产品创意岗位中已经 native 化的内容 skill 接入稳定 workspace artifact 体系。
- 让趋势捕捉和标题 CTR 评分成为可查询、可预览、可导出的创意交付物。
- 修正已有创意内容 skill 输出契约，让短视频脚本、种草文案、IP定位、内容排期、直播脚本和产品软文的既有 artifact 映射真实生效。

涉及范围：

- 产品内核：创意岗位从“当轮 skill 结果”推进到“可复用的内容交付物”。
- 后端 agent：`creative_*` skill 输出补齐 `status=success`、`has_data`、`adapter_status`、`data_readiness` 和 `summary`，继续保留缺必填参数时的 validation error。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增支持 `creative_trend_report`、`creative_title_ctr_score`。
- API/SSE 文档：同步 creative artifact 类型、默认估算披露和缺输入不创建假交付物的语义。

用户可感知变化：

- 用户让创意岗位做短视频脚本、种草文案、IP定位、内容排期、直播脚本、产品软文、趋势方向或标题 CTR 评分时，成功结果可以在当前 workspace 真实新增对应创意交付物。
- 这些交付物沿用现有 artifact 列表、详情、预览和导出链路。
- 如果使用默认商品、排期或直播参数，结果会通过 `data_readiness=calculated_from_defaults` 明确告知前端和用户。
- 如果缺少必填参数，例如趋势捕捉缺少 `category` 或标题评分缺少 `title`，后端返回 validation error，不会创建假交付物。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent creative_ -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_creative_ -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`
