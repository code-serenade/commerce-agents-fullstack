# Post 8-34: Service Insight Artifact Materialization

## 1. Background

Post 8-33 persisted the core service support playbooks:

- `service_ticket_handler`
- `service_faq_playbook`
- `service_escalation_flow`
- `service_dsr_improvement`

The remaining native-ready service skills still produce useful structured output, but they are not yet materialized as reusable workspace artifacts:

- `service_return_handler`
- `service_nps_analyzer`
- `service_sentiment_analyzer`
- `service_nps_driver_analysis`

## 2. Goal

Persist successful service after-sales and customer insight outputs as real workspace artifacts:

```text
service insight skill_result -> WorkspaceApi::materialize_agent_content_artifact -> workspace_artifact
```

This keeps the feature inside the teammate-provided framework:

- Rust Agent skill executor
- service `WorkspaceApi`
- existing `workspace_artifacts`
- existing `workspace_artifact_created` SSE event
- existing `API_DOCS`

## 3. Scope

This slice covers:

- `service_return_handler` -> `artifactType=service_return_plan`
- `service_nps_analyzer` -> `artifactType=service_nps_report`
- `service_sentiment_analyzer` -> `artifactType=service_sentiment_report`
- `service_nps_driver_analysis` -> `artifactType=service_nps_driver_report`

No new database table, HTTP route, frontend-only state, or separate analytics service is introduced.

## 4. Agent Output Contract

Successful outputs should include:

- `status=success`
- `has_data=true`
- `data_readiness=user_provided`, `user_text_provided`, `user_batch_provided`, or `calculated_from_defaults`
- `adapter_status=manual_input`
- `summary`

Default sample data must be disclosed with:

- `data_readiness=calculated_from_defaults`

This prevents the frontend from presenting deterministic examples as live platform metrics.

## 5. Service Design

Extend `build_content_artifact_from_skill_output` in `WorkspaceApi` with service insight mappings.

The existing service method continues to:

1. check `status=success`
2. check supported `skillId`
3. build markdown from structured JSON output
4. write `workspace_artifacts`
5. update `agent_skill_runs.metadata.contentArtifactId`
6. return the artifact to web-server for `workspace_artifact_created`

## 6. Artifact Contract

Common fields:

- `storageKind=inline`
- `format=markdown`
- `mimeType=text/markdown`
- `source=agent_session`
- `metadata.kind=agent_generated_content`
- `metadata.contentCategory=service`

Supported artifact types:

- `service_return_plan`
- `service_nps_report`
- `service_sentiment_report`
- `service_nps_driver_report`

## 7. User-Visible Behavior

Users can ask the service worker for return handling, NPS/CSAT analysis, sentiment triage, or satisfaction driver analysis. When the backend succeeds, the result can become a workspace deliverable and later appear in the existing artifact list/detail/export flow.

If NPS or driver analysis uses default sample data, the artifact keeps the disclosure so the frontend can avoid presenting it as live store metrics.

## 8. Non-Goals

This slice does not:

- connect live platform customer service systems
- create a ticket database
- read live survey systems
- perform platform write actions
- add frontend-only fake reports

## 9. Validation

Required checks:

- `cargo fmt`
- targeted agent tests for service insight success output contracts
- targeted service test for service insight artifact mapping
- `cargo check -p agent`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p agent -p service -p web-server`
