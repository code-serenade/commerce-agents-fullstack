# Agent Chat Workspace Context UI Fix

## Requirement Summary

The chat page should be usable for a product demo:

- only one left sidebar collapse control should be visible;
- the right virtual employee workspace should not cover the answer area;
- product context selection must be visible and clearly manual;
- process cards should appear before the assistant answer and collapse after the answer finishes.

## Frontend Design

- Reuse the existing `SidebarShell`, `TopBar`, and page-local agent chat components.
- Disable the top-bar sidebar trigger in the agent chat shell because `SidebarShell` already owns the sidebar collapse button.
- Render the virtual employee workspace as a real split column on wide screens and a stacked pane on narrow screens, not as a floating overlay.
- Keep product selection inside the existing task settings panel, but add a visible "choose product / no product selected" entry in the composer settings summary.
- Insert the inline process card immediately after the latest user message so it appears before the streamed answer.
- Keep the process card expanded while streaming, then auto-collapse it when the run finishes.

## API Mapping

The frontend sends product context only through the existing `metadata.context.productIds` and `metadata.context.contextRefs` fields. If the user does not select any product, both arrays remain empty.

## Verification

- `pnpm -C frontend/apps/web lint`
- `pnpm -C frontend/apps/web rules`
- `pnpm -C frontend/apps/web test`
- `pnpm -C frontend/apps/web build`
- browser smoke test on `/commerce`
