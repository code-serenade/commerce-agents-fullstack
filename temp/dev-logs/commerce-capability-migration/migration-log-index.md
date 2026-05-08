# Commerce Capability Migration Log Index

## Rule

Every completed migration step must add a dedicated file in this directory:

```text
NNN-<capability>-completed.md
```

Each completion file must record:

- Scope completed.
- Files changed.
- APIs added or changed.
- Validation commands and results.
- Confirmation that `backend/crates/agent/` was not modified.
- Follow-up blockers.

## Current Status

### 000: Planning

Status: completed.

Files:

- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_phase_01_products.md`
- `temp/dev-logs/commerce-capability-migration/agent-crate-blockers.md`

Next step:

- Phase 01 Product Core implementation.

### 001: Product Core

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/001-product-core-completed.md`

Next step:

- Phase 02 Product Knowledge.

### 002: Product Knowledge

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/002-product-knowledge-completed.md`

Next step:

- Phase 03 Product Materials/Assets.

### 003: Product Materials And Assets

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/003-product-materials-assets-completed.md`

Next step:

- Phase 04 Competitor Monitoring.

### 004: Competitor Monitoring

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/004-competitor-monitoring-completed.md`

Next step:

- Phase 05 Campaigns.

### 005: Campaigns

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/005-campaigns-completed.md`

Next step:

- Phase 06 Workspace/Execution List.

### 006: Workspace And Execution List

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/006-workspace-execution-completed.md`

Next step:

- Phase 07 Platform Connect.

### 007: Platform Connect

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/007-platform-connect-completed.md`

Next step:

- Phase 08 Approval And Audit.

### 008: Approval And Audit

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/008-approval-audit-completed.md`

Next step:

- Phase 09 Long Tasks, Briefing, Alerts.

### 009: Long Tasks, Briefing, Notifications, And Alerts

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/009-long-tasks-briefing-alerts-completed.md`

Next step:

- Phase 10 Capability Package Governance.

### 010: Capability Package Governance

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/010-package-governance-completed.md`

Next step:

- User review before starting the next migration phase.

### 011: Frontend Backend Capability Adaptation

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/011-frontend-backend-capability-adaptation-completed.md`

Next step:

- Focused frontend/backend alignment fixes.

### 012: Frontend Product Backend Alignment

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/012-frontend-product-backend-alignment.md`

### 013: Frontend Product Update Actions

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/013-frontend-product-update-actions.md`

### 014: Frontend Detail Read Alignment

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/014-frontend-detail-read-alignment.md`

### 015: Cloud Ollama LLM Config Alignment

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/015-cloud-ollama-llm-config-alignment.md`

### 016: Frontend Backend Smoke Hardening

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/016-frontend-backend-smoke-hardening.md`

### 017: QA Core Commerce Agent Integration

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/017-qa-core-commerce-agent-integration.md`
018 - Agent chat frontend runtime alignment
  File: temp/dev-logs/commerce-capability-migration/018-agent-chat-frontend-runtime-alignment.md
  Summary: Reconnected the dedicated chat shell/provider, made the frontend SSE adapter consume real backend stream events, mapped simple worker events into the virtual employee workspace, refreshed product positioning, and improved right-side collapse/expand behavior.

### 019: Agent Chat Product Experience Alignment

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/019-agent-chat-product-experience-alignment.md`

Summary:

- Reworked the real user path around Q&A first, usable shell navigation, draft-before-login behavior, composer business context, right-side process panel controls, and authenticated SSE validation.

### 020: Frontend Backend Connectivity Hardening

Status: completed.

Archive:

- `temp/dev-logs/commerce-capability-migration/020-frontend-backend-connectivity-hardening.md`

Summary:

- Fixed the product detail frontend adapter mismatch, added authenticated agent SSE smoke coverage with real cloud model text, added SSE token-refresh retry, and confirmed `backend/crates/agent/` remained untouched.
