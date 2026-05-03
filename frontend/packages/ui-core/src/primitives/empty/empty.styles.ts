import { cva } from "class-variance-authority"

export const emptyClassNames = {
  slot0:
    "flex w-full min-w-0 flex-1 flex-col items-center justify-center gap-4 rounded-xl border-dashed p-6 text-center text-balance",
  slot4: "flex max-w-sm flex-col items-center gap-2",
  slot5: "font-heading text-sm font-medium tracking-tight",
  slot6:
    "text-sm/relaxed text-muted-foreground [&>a]:underline [&>a]:underline-offset-4 [&>a:hover]:text-primary",
  slot7:
    "flex w-full max-w-sm min-w-0 flex-col items-center gap-2.5 text-sm text-balance",
  slot1:
    "mb-2 flex shrink-0 items-center justify-center [&_svg]:pointer-events-none [&_svg]:shrink-0",
  slot2: "bg-transparent",
  slot3:
    "flex size-8 shrink-0 items-center justify-center rounded-lg bg-muted text-foreground [&_svg:not([class*='size-'])]:size-4",
} as const

export const emptyMediaVariants = cva(emptyClassNames.slot1, {
  variants: {
    variant: {
      default: emptyClassNames.slot2,
      icon: emptyClassNames.slot3,
    },
  },
  defaultVariants: {
    variant: "default",
  },
})
