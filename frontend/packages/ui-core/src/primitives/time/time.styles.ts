export const timeClassNames = {
  root: "flex items-center gap-2",
  column:
    "relative h-[calc(var(--time-item-height)*var(--time-visible-rows))] min-w-22 overflow-y-auto rounded-[min(var(--radius-xl),1.25rem)] bg-[var(--surface)] snap-y snap-mandatory overscroll-contain [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden",
  item: "flex h-[var(--time-item-height)] snap-center items-center justify-center px-3 text-lg font-semibold text-muted-foreground select-none transition-colors aria-selected:text-foreground aria-selected:opacity-100 opacity-45 pointer-events-none",
  selection:
    "pointer-events-none absolute inset-x-1 top-1/2 h-[calc(var(--time-item-height)-2px)] -translate-y-1/2 rounded-[min(var(--radius-lg),1rem)] border border-border bg-[var(--surface-hover)]/70 shadow-sm backdrop-blur-sm",
} as const
