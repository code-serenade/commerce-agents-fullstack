import { cva } from "class-variance-authority"

export const buttonVariants = cva(
  "group/button inline-flex shrink-0 items-center justify-center rounded-lg border border-transparent bg-clip-padding text-sm font-medium whitespace-nowrap transition-all outline-none select-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 active:not-aria-[haspopup]:translate-y-px disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
  {
    variants: {
      variant: {
        default:
          "bg-[var(--surface)] text-[var(--surface-foreground,var(--foreground))] hover:bg-[var(--surface-hover,var(--surface))] disabled:bg-[var(--surface-disabled,var(--surface))]",
        primary:
          "bg-[var(--primary)] text-[var(--primary-foreground,var(--background))] hover:bg-[var(--primary-hover,var(--primary))] disabled:bg-[var(--primary-disabled,var(--primary))]",
        secondary:
          "bg-[var(--secondary)] text-[var(--secondary-foreground,var(--foreground))] hover:bg-[var(--secondary-hover,var(--secondary))] disabled:bg-[var(--secondary-disabled,var(--secondary))]",
        destructive:
          "bg-[var(--destructive)] text-[var(--destructive-foreground,var(--primary-foreground,var(--background)))] hover:bg-[var(--destructive-hover,var(--destructive))] focus-visible:border-destructive/40 focus-visible:ring-destructive/20 disabled:bg-[var(--destructive-disabled,var(--destructive))]",
        outline:
          "border-[var(--primary-outline,var(--primary))] bg-[var(--surface)] text-[var(--primary-outline,var(--primary))] hover:bg-[var(--surface-hover,var(--surface))]",
        ghost:
          "bg-[var(--background)] text-[var(--foreground)] hover:bg-[var(--surface)] hover:text-[var(--foreground)] aria-expanded:bg-[var(--surface)] aria-expanded:text-[var(--foreground)]",
        link: "h-auto bg-transparent p-0 text-[var(--foreground)] underline-offset-4 hover:text-[var(--foreground)] hover:underline active:text-[var(--foreground)]",
      },
      size: {
        default:
          "h-8 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-2 has-data-[icon=inline-start]:pl-2",
        xs: "h-6 gap-1 rounded-[min(var(--radius-md),10px)] px-2 text-xs in-data-[slot=button-group]:rounded-lg has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 [&_svg:not([class*='size-'])]:size-3",
        sm: "h-7 gap-1 rounded-[min(var(--radius-md),12px)] px-2.5 text-[0.8rem] in-data-[slot=button-group]:rounded-lg has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 [&_svg:not([class*='size-'])]:size-3.5",
        lg: "h-9 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-3 has-data-[icon=inline-start]:pl-3",
        icon: "size-8",
        "icon-xs":
          "size-6 rounded-[min(var(--radius-md),10px)] in-data-[slot=button-group]:rounded-lg [&_svg:not([class*='size-'])]:size-3",
        "icon-sm":
          "size-7 rounded-[min(var(--radius-md),12px)] in-data-[slot=button-group]:rounded-lg",
        "icon-lg": "size-9",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)
