# Post 8-78 Data Channel ROI Product Ref

## Requirement Summary

Continue backend-first migration of old product capabilities. This slice extends the native `data_channel_roi` skill so product-bound channel ROI / attribution-style review can carry the selected-product reference:

```text
product_ref = product:{uuid}
```

The skill still only uses explicit channel metrics from the current turn. It must not pretend to fetch live Taobao, Douyin, Qianchuan, or other platform data.

## Scope

- Agent data skill: `data_channel_roi`
- Data worker autosuggest for channel ROI / channel performance intent
- SSE/API documentation
- Migration logs and matrix

## Out Of Scope

- No database migration.
- No HTTP API route change.
- No frontend code change.
- No official platform adapter integration.
- No fake live channel metrics.

## Data Contract

`data_channel_roi` accepts:

- `product_ref?: string`
- `days?: integer`
- `include_organic?: boolean`
- `channels?: array<object>`

Each channel remains:

```json
{"name":"douyin","gmv":120000,"ad_spend":30000,"orders":600,"uv":20000}
```

`product_ref` is an identity field only. ROI calculations still depend on explicit channel metrics or future service/adapter injected metrics.

## Behavior

- Successful output includes `商品引用`.
- Deferred output already echoes `received_params`; when a `product_ref` is supplied it remains visible there.
- Worker autosuggest passes `product_ref` only from explicit `context.selected_summary`.
- Without selected product context, autosuggest does not add an empty `product_ref`.

## API / Auth

No new API route. Existing authenticated Agent SSE flow is unchanged:

```text
POST /api/sse/agent/session/stream
```

The field appears inside existing `skill_requested.input` and `skill_result.output`.

## Validation

- `cargo fmt --all`
- `cargo test -p agent data_channel_roi -- --nocapture`
- `cargo test -p agent selected_product_context_feeds_channel_roi_ref -- --nocapture`
- `cargo test -p agent channel_roi_without_selection_has_no_empty_product_ref -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
