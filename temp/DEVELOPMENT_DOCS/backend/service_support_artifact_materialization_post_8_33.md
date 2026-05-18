# Post 8-33: Service Support Artifact Materialization

## 1. Background

The migration matrix marks the old product's core service support skills as native-ready in the new product:

- `service_ticket_handler`
- `service_faq_playbook`
- `service_escalation_flow`
- `service_dsr_improvement`

These skills already run inside the existing Rust Agent framework, but their successful outputs are not yet persisted as workspace deliverables.

## 2. Goal

Persist successful core service support skill outputs as real workspace artifacts:

```text
service skill_result -> WorkspaceApi::materialize_agent_content_artifact -> workspace_artifact
```

This keeps the feature inside the teammate-provided framework:

- Rust Agent skill executor
- service `WorkspaceApi`
- existing `workspace_artifacts`
- existing `workspace_artifact_created` SSE event
- existing `API_DOCS`

## 3. Scope

This slice covers:

- `service_ticket_handler` -> `artifactType=service_ticket_playbook`
- `service_faq_playbook` -> `artifactType=service_faq_playbook`
- `service_escalation_flow` -> `artifactType=service_escalation_playbook`
- `service_dsr_improvement` -> `artifactType=service_dsr_improvement_plan`

No new database table, HTTP route, frontend-only state, or separate support document system is introduced.

## 4. Agent Output Contract

Successful outputs should include:

- `status=success`
- `has_data=true`
- `data_readiness=user_provided`, `user_policy_provided`, `general_sop_defaults`, or `calculated_from_defaults`
- `adapter_status=manual_input`
- `summary`

`service_dsr_improvement` may use deterministic default sample scores when the user does not provide DSR inputs, but it must disclose this with:

- `data_readiness=calculated_from_defaults`

## 5. Service Design

Extend `build_content_artifact_from_skill_output` in `WorkspaceApi` with service support mappings.

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

- `service_ticket_playbook`
- `service_faq_playbook`
- `service_escalation_playbook`
- `service_dsr_improvement_plan`

## 7. User-Visible Behavior

Users can ask the service worker for complaint handling, FAQ playbooks, escalation flows, or DSR improvement. When the backend successfully computes the result, it can become a workspace deliverable and later appear in the existing artifact list/detail/export flow.

If DSR uses default sample scores, the artifact keeps the disclosure so the frontend can avoid presenting it as live store metrics.

## 8. Non-Goals

This slice does not:

- connect live platform customer service tickets
- create a ticket database
- perform platform write actions
- add a separate support center UI
- add frontend-only fake support documents

## 9. Validation

Required checks:

- `cargo fmt`
- targeted agent tests for service support success output contracts
- targeted service test for service support artifact mapping
- `cargo check -p agent`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p agent -p service -p web-server`
