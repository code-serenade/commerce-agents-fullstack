# 020 Frontend Backend Connectivity Hardening

Date: 2026-05-08

## Scope

- Hardened the real frontend/backend path for commerce product detail and agent Q&A.
- Fixed the product detail frontend adapter so `GET /api/app/products/{productId}` unwraps the backend `ProductDetailResponse.product` payload before giving it to page code.
- Extended the smoke verification script so it no longer only creates conversations; it now also calls the authenticated SSE agent stream and verifies real model text is returned.
- Added one retry path for agent SSE when the access token expires: a `401` response now uses the stored refresh token to obtain a new access token and retries the stream once.

## Files Changed

- `frontend/apps/web/src/api/commerce.ts`
- `frontend/apps/web/src/api/agent.ts`
- `temp/smoke-commerce-connectivity.mjs`
- `temp/dev-logs/commerce-capability-migration/020-frontend-backend-connectivity-hardening.md`
- `temp/dev-logs/commerce-capability-migration/migration-log-index.md`

## APIs Added Or Changed

- No backend API was added or changed.
- Frontend adapter alignment:
  - `getProductApi()` now maps backend `data.product` to the direct `ProductResponse` shape expected by product UI code.
- Existing APIs validated:
  - `POST /auth/session/login`
  - `GET /auth/user/me`
  - `GET /api/app/me/permissions`
  - Product, product knowledge, product asset, product material, competitor, campaign, workspace, execution queue, platform, channel, approval, audit, long task, briefing, notification, alert, package governance, and conversation routes covered by smoke script.
  - `POST /api/sse/agent/session/stream` verified with authenticated token and cloud Ollama-backed model output.

## Validation

- `node temp/smoke-commerce-connectivity.mjs`: passed, 58/58 checks, including `agent.stream.answer` with returned text `2+2 等于 4。`
- `pnpm -C apps/web rules`: passed.
- `pnpm -C apps/web lint`: passed with existing Fast Refresh warnings in commerce shared UI files; no errors.
- `pnpm -C apps/web build`: passed.
- `pnpm -C apps/web test`: passed; current web app has no test files.
- `pnpm rules`: passed.
- `git status --short backend/crates/agent`: no changes.

## Agent Crate Boundary

No files under `backend/crates/agent/` were modified.

## Result

The current evidence does not support the idea that the backend business APIs or the cloud Ollama-backed Q&A core are globally disconnected. The verified gap is more precise:

- Core backend routes are reachable through the frontend dev proxy.
- Authenticated agent SSE returns real model text.
- At least one product detail adapter mismatch was real and is now fixed.
- The remaining product risk is mainly frontend assembly and UX polish: page state, default values, error display, and richer user-flow integration can still make working backend capabilities feel missing.

## Follow-up

- Add browser-level user-flow tests when a browser automation tool is available in this environment.
- Continue checking page-by-page rendering and action wiring, prioritizing `/commerce` Q&A, product detail, workspace execution, platform connection, approvals, operations, and packages.
- Keep any runtime strategy-routing changes that require `backend/crates/agent/` in `agent-crate-blockers.md` instead of implementing them under the hard boundary.
