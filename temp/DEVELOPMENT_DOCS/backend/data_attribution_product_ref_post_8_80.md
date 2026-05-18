# Post 8-80 Data Attribution Product Ref

## Requirement Summary

Continue backend-first migration of old product capabilities. This slice extends the native `data_attribution_analysis` skill so product-bound paid-channel attribution can carry the selected-product reference:

```text
product_ref = product:{uuid}
```

It also improves natural-language reachability for explicit attribution intent. Channel ROI and channel attribution are related but not identical:

- Channel ROI reviews spend efficiency.
- Channel attribution compares contribution models across channels.

## Scope

- Agent data skill: `data_attribution_analysis`
- Data worker autosuggest for channel attribution / multi-touch attribution intent
- SSE/API documentation
- Migration logs and matrix

## Out Of Scope

- No database migration.
- No HTTP API route change.
- No frontend code change.
- No official platform adapter integration.
- No fake live ad platform attribution metrics.

## Data Contract

`data_attribution_analysis` accepts:

- `product_ref?: string`
- `days?: integer`
- `attribution_model?: string`
- `channels?: array<object>`

Each channel remains:

```json
{"name":"抖音","gmv":180000,"uv":50000,"conversion_rate":3.2,"ad_spend":48000}
```

`product_ref` is an identity field only. Attribution calculations still depend on explicit channel metrics or future service/adapter injected metrics.

## Behavior

- Successful output includes `商品引用`.
- Worker autosuggest routes explicit `渠道归因` / `多触点归因` / `归因模型` intent to `data_attribution_analysis`.
- Worker autosuggest continues routing `渠道ROI` / `渠道投产` intent to `data_channel_roi`.
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
- `cargo test -p agent data_attribution_analysis -- --nocapture`
- `cargo test -p agent autosuggests_attribution_analysis_with_explicit_channel_metrics -- --nocapture`
- `cargo test -p agent selected_product_context_feeds_attribution_analysis_ref -- --nocapture`
- `cargo test -p agent attribution_analysis_without_selection_has_no_empty_product_ref -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
