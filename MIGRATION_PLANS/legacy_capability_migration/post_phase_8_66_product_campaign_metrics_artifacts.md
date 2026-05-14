# Post 8-66：商品 / 活动指标报告 artifact 沉淀

状态：done

## 目标

把 Post 8-65 已迁好的 `query_product_metrics` 和 `query_campaign_metrics` 从“只返回 skill_result”推进到“成功时可沉淀为工作区报告”。

## 修改范围

- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 支持：
  - `query_product_metrics` -> `data_product_metrics_report`
  - `query_campaign_metrics` -> `data_campaign_metrics_report`
- 后端测试：新增成功物化和 deferred 不物化测试。
- API 文档：同步 `API_DOCS/app.md` 和 `API_DOCS/sse.md`。
- 迁移文档：同步矩阵、开发日志、验收记录和总计划。

## 真实性边界

- 只有 `output.status=success` 才创建 artifact。
- 缺少商品指标时的 `product_metrics_required` 不创建 artifact。
- 缺少活动指标时的 `campaign_metrics_required` 不创建 artifact。
- 不默认读取预设商品。
- 不伪造淘宝、抖音、千川或其它平台实时指标。

## 用户可感知变化

用户在问答中明确提供商品 / 活动指标并触发对应 skill 后，后端现在可以在工作区保存一份可复用的商品指标报告或活动指标报告。

后续前端可以在交付物列表、过程面板、历史运行详情和导出入口展示这些报告。

## 验证

- `cargo test -p service content_artifact_builds_from_product_and_campaign_metrics_skill_output -- --nocapture`
- `cargo test -p service content_artifact_ignores_deferred_product_and_campaign_metrics -- --nocapture`
- `cargo check -p service`
- `cargo clippy -p service`
- `cargo fmt --all --check`
- `git diff --check`
