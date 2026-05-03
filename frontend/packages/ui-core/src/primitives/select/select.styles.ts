export const selectGroupClassName =
  "scroll-my-1 space-y-0.5 p-1 [&:not(:first-child)]:mt-1"

export const selectTriggerClassName =
  "flex w-fit items-center justify-between gap-1.5 rounded-lg py-2 pr-2 pl-2.5 text-sm whitespace-nowrap transition-colors outline-none select-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 data-placeholder:text-muted-foreground data-[size=default]:h-8 data-[size=sm]:h-7 data-[size=sm]:rounded-[min(var(--radius-md),10px)] *:data-[slot=select-value]:line-clamp-1 *:data-[slot=select-value]:flex *:data-[slot=select-value]:items-center *:data-[slot=select-value]:gap-1.5 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"

export const selectTriggerSurfaceClassName =
  "border border-[var(--select-border)] bg-transparent text-foreground hover:bg-[var(--surface)] dark:bg-input/30 dark:hover:bg-[var(--surface)]"

// Backward-compatible alias for external imports.
export const selectTriggerVariantClassNames = {
  default: selectTriggerSurfaceClassName,
  primary: selectTriggerSurfaceClassName,
  destructive: selectTriggerSurfaceClassName,
} as const

export const selectTriggerIconClassName =
  "pointer-events-none inline-flex size-6 items-center justify-center rounded-full bg-[var(--surface)] text-[var(--select-foreground)]"

export const selectContentBaseClassName =
  "relative z-50 max-h-[min(20rem,var(--radix-select-content-available-height))] min-w-36 origin-(--radix-select-content-transform-origin) overflow-x-hidden overflow-y-auto rounded-lg border border-[var(--select-border)] bg-[var(--select)] p-1 text-[var(--select-foreground)] shadow-md ring-1 ring-[var(--select-ring)]/10 duration-100 data-[align-trigger=true]:animate-none data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95"

// Backward-compatible alias for external imports.
export const selectContentVariantClassNames = {
  default: "",
  primary: "",
  destructive: "",
} as const

export const selectContentPopperOffsetClassName =
  "data-[side=bottom]:translate-y-1 data-[side=left]:-translate-x-1 data-[side=right]:translate-x-1 data-[side=top]:-translate-y-1"

export const selectViewportClassName =
  "data-[position=popper]:w-full data-[position=popper]:min-w-(--radix-select-trigger-width)"

export const selectLabelClassName = "px-1.5 py-1 text-xs text-muted-foreground"

export const selectItemClassName =
  "relative flex w-full cursor-default items-center gap-1.5 rounded-md py-1 pr-1.5 pl-8 text-sm outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 *:[span]:last:flex *:[span]:last:items-center *:[span]:last:gap-2"
export const selectItemStateClassName =
  "hover:bg-[var(--select-item-hover)] hover:text-[var(--select-item-hover-foreground)] data-[highlighted]:bg-[var(--select-item-hover)] data-[highlighted]:text-[var(--select-item-hover-foreground)] focus:bg-[var(--select-item-hover)] focus:text-[var(--select-item-hover-foreground)]"

// Backward-compatible alias for external imports.
export const selectItemVariantClassNames = {
  default: selectItemStateClassName,
  primary: selectItemStateClassName,
  destructive: selectItemStateClassName,
} as const

export const selectItemIndicatorContainerClassName =
  "pointer-events-none absolute left-2 flex size-4 items-center justify-center"

export const selectSeparatorClassName =
  "pointer-events-none -mx-1 my-1 h-px bg-[var(--select-border)]"

export const selectScrollButtonClassName =
  "z-10 flex cursor-default items-center justify-center bg-[var(--select)] py-1 text-[var(--select-foreground)] [&_svg:not([class*='size-'])]:size-4"
