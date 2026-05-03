# Components Architecture

```text
components/
  ui/                # Stable semantic UI API (variant-driven)
    button.tsx
    input.tsx
    card.tsx
    index.ts
  hooks/             # Reusable behavior-only hooks/utilities
    useComposedRefs.ts
  utils/             # Pure helpers
    cn.ts
  examples/          # Usage demos for developers/AI
    DesignSystemExample.tsx
```

## Rules

1. `ui/*` exposes typed, predictable props (`variant`, `size`, `state`), not stringly class contracts.
2. Tailwind remains implementation detail inside UI primitives.
3. Business pages should consume semantic UI APIs, not large Tailwind strings.
