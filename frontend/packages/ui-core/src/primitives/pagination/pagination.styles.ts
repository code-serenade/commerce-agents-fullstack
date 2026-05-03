export const paginationClassNames = {
  slot1: "mx-auto flex w-full justify-center",
  slot2: "flex items-center gap-0.5",
  slot3:
    "bg-transparent hover:bg-[var(--surface-hover)] aria-expanded:bg-[var(--surface-hover)] data-[active=true]:bg-[var(--surface-active)] aria-disabled:pointer-events-none aria-disabled:opacity-40",
  slot4:
    "flex size-8 items-center justify-center [&_svg:not([class*='size-'])]:size-4",
  slot5: "sr-only",
} as const
