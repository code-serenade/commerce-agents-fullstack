# Agent Chat Explicit Product Context Fix

## Requirement Summary

The agent chat must distinguish two backend paths:

- No product selected: answer as a generic ecommerce question and do not inject any saved product facts, product knowledge, product assets, or product materials.
- Product selected by the user: validate the selected product ownership and inject only that product's backend context.

This prevents a product such as `千乐咖啡` from being used as default context just because the question contains a matching keyword like `咖啡`.

## Backend Design

- Keep the existing SSE API shape.
- Treat `metadata.context.productIds` and `metadata.context.contextRefs` as the only product opt-in signals.
- When those fields resolve to no product ids, local product RAG returns an empty, non-injected context.
- In the agent SSE path, workspace context summary and context pack must not fall back to a workspace-bound product unless the product is explicitly selected in this turn.

## API Impact

No new route or request field is required. The semantics of `/api/sse/agent/session/stream` are clarified:

- empty product selection means no product-specific grounding;
- explicit product ids continue to use the existing validation and context-loading path.

`API_DOCS/sse.md` is updated with this rule.

## Verification

- Run focused backend tests for agent context and SSE metadata.
- Run frontend app checks after UI alignment.
