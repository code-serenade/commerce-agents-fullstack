import { cva } from "class-variance-authority"

export const badgeClassNames = {
  slot1:
    "group/badge inline-flex h-5 w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-4xl border border-transparent px-2 py-0.5 text-xs font-medium whitespace-nowrap transition-all focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&>svg]:pointer-events-none [&>svg]:size-3!",
  slot2: "bg-primary text-primary-foreground [a]:hover:bg-primary/80",
  slot3: "bg-secondary text-secondary-foreground [a]:hover:bg-secondary/80",
  slot4:
    "bg-destructive/10 text-destructive focus-visible:ring-destructive/20 dark:bg-destructive/20 dark:focus-visible:ring-destructive/40 [a]:hover:bg-destructive/20",
  slot5:
    "border-border text-foreground [a]:hover:bg-muted [a]:hover:text-muted-foreground",
  slot6: "hover:bg-muted hover:text-muted-foreground dark:hover:bg-muted/50",
  slot7: "text-primary underline-offset-4 hover:underline",
} as const

export const badgeVariants = cva(badgeClassNames.slot1, {
  variants: {
    variant: {
      default: badgeClassNames.slot2,
      secondary: badgeClassNames.slot3,
      destructive: badgeClassNames.slot4,
      outline: badgeClassNames.slot5,
      ghost: badgeClassNames.slot6,
      link: badgeClassNames.slot7,
    },
  },
  defaultVariants: {
    variant: "default",
  },
})
