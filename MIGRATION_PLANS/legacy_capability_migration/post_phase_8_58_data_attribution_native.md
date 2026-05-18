# Post 8-58: data_attribution_analysis Native Skill

## Status

Done.

## Purpose

Migrate old product channel attribution analysis into the new product Rust Agent skill framework.

## Backend Changes

- `data_attribution_analysis` is no longer a deferred placeholder.
- The skill accepts explicit `channels` data and computes:
  - last-touch attribution
  - linear attribution
  - time-decay attribution
  - position-based attribution
- Missing, single-channel, or zero-GMV data returns `status=deferred`.
- The skill does not read external platform data directly and does not fake platform metrics.

## Artifact Changes

Successful output can now become:

- `artifactType=data_attribution_report`

Only `status=success` output creates a workspace artifact.

## User-Visible Meaning

After frontend connects the artifact and process panel, users can provide multi-channel metrics and get a durable channel attribution report. If they do not provide channel data, the system asks for data instead of pretending to have read platform metrics.

## Validation

- [x] `cargo fmt --all`
- [x] `cargo fmt --all --check`
- [x] `cargo test -p agent --lib data_attribution_analysis -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_data_advanced_skill_output -- --nocapture`
- [x] `cargo check -p agent -p service`
- [x] `cargo clippy -p agent -p service`

## Remaining Gaps

- Real platform/warehouse adapters remain deferred.
- `data_comprehensive_diagnosis` should be migrated after this because it can now consume attribution results as one sub-diagnosis.
- Frontend still needs to display attribution details and artifact links in the workbench.
