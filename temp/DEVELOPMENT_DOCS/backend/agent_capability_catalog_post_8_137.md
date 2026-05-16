# Post 8-137 Agent Capability Catalog Backend Design

## Goal

Build a backend-first contract for the frontend "功能导航 / 虚拟员工能力介绍" panel.

The frontend currently needs to explain roles and skills, but that should not be hard-coded in UI. The source of truth should be the existing Agent runtime framework:

- `RoleCatalog::configured()`
- `RoleSkillCatalog::staged()`
- `SkillRegistry::staged()`
- skill execution metadata from the Agent runtime

## Endpoint

```text
GET /api/app/agent/capability-catalog
```

Authentication:

- Valid App JWT.
- The current app user must exist and be enabled, using the same `CommercePlaybookApi::ensure_current_user_enabled` boundary as `/api/app/agent/runtime-options`.

## Response Shape

The endpoint returns:

- catalog source and version marker
- summary counts
- role list
- each role's domains, keywords, preferred actions, required / optional capabilities
- each role's usable skills
- each skill's id, display name, description, category, priority, execution mode, risk level, approval requirement, adapter requirements, tags, and input fields

## Architecture Rules

- No DB access in `web-server`.
- `web-server` only calls service for current-user validation.
- Capability data comes from the Agent crate public API.
- No frontend fallback data is required.
- No LLM call is made.
- No new database table is required.

## User-Visible Value

After frontend adaptation, users can open "功能导航" and see the real virtual employees and the real backend skills they can trigger. This lets the UI explain what the product can do without inventing unsupported buttons.

## Validation

Run:

```text
cargo fmt --all
cargo test -p web-server agent_capability_catalog
cargo check -p agent -p web-server
cargo clippy -p agent -p web-server
```
