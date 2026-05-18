# Post 8-29: Data Dashboard Native Skill

## Status

Done.

## Goal

Migrate the old product's data dashboard capability into the new Agent skill framework as a real backend native skill.

## Backend Change

`data_dashboard` now produces a deterministic structured dashboard:

- core metric cards
- growth / efficiency / risk sections
- operating health score
- alerts
- next actions
- explicit data readiness

When user metrics are provided, the skill returns:

```text
status=success
has_data=true
adapter_status=manual_input
```

When metrics are missing, it still returns the dashboard structure, but marks it as:

```text
has_data=false
adapter_status=store_metrics_required
```

This prevents the frontend from treating sample baselines as live platform data.

## Trigger Change

The data worker autosuggest path now routes dashboard / data overview / operating overview / core metric phrasing to `data_dashboard`.

## User-Visible Change

Users asking for "店铺大盘", "数据看板", "经营概览", or "核心经营指标" can now get a real structured backend dashboard instead of a deferred placeholder.

The product still does not claim to have queried live platform metrics unless those metrics are supplied or a later adapter is connected.

## Remaining Work

Live store metric ingestion remains a later adapter/database slice:

- platform metric adapter
- store metric persistence
- dashboard history and comparison by product / campaign / workspace
