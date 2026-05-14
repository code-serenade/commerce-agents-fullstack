# Post 8-67：店铺指标报告 artifact 沉淀

状态：done

## 目标

把 `query_store_metrics` 从“能返回结构化 skill_result”推进到“成功时可沉淀为工作区报告”。

这样店铺、商品、活动三层指标查询都能进入新产品 workspace artifact 体系。

## 修改范围

- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 支持：
  - `query_store_metrics` -> `data_store_metrics_report`
- 后端测试：新增成功物化和 deferred 不物化测试。
- API 文档：同步 `API_DOCS/app.md` 和 `API_DOCS/sse.md`。
- 迁移文档：同步矩阵、开发日志、验收记录和总计划。

## 真实性边界

- 只有 `output.status=success` 才创建 artifact。
- 缺少店铺指标时的 `store_metrics_required` 不创建 artifact。
- 不伪造淘宝、抖音或其它平台实时店铺指标。
- 不新增前端假状态。

## 用户可感知变化

用户在问答中明确提供店铺 GMV、订单、UV、广告费、退款、毛利或平台拆分等指标并触发 `query_store_metrics` 后，后端现在可以在工作区保存一份可复用的店铺指标报告。

## 验证

- `cargo test -p service content_artifact_builds_from_store_metrics_skill_output -- --nocapture`
- `cargo test -p service content_artifact_ignores_deferred_store_metrics -- --nocapture`
- `cargo check -p service`
- `cargo clippy -p service`
- `cargo fmt --all --check`
- `git diff --check`
