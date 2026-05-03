import { cva } from "class-variance-authority"

export const alertVariants = cva(
  "group/alert relative grid w-full gap-0.5 rounded-lg border bg-[var(--surface)] px-2.5 py-2 text-left text-sm has-data-[slot=alert-action]:relative has-data-[slot=alert-action]:pr-18 has-[>svg]:grid-cols-[auto_1fr] has-[>svg]:gap-x-2 *:[svg]:row-span-2 *:[svg]:translate-y-0.5 *:[svg:not([class*='size-'])]:size-4",
  {
    variants: {
      variant: {
        default:
          "border-[color-mix(in_oklab,var(--info)_35%,var(--border))] text-[var(--info)] *:[svg]:text-current",
        success:
          "border-[color-mix(in_oklab,var(--success)_35%,var(--border))] text-[var(--success)] *:[svg]:text-current",
        warning:
          "border-[color-mix(in_oklab,var(--warning)_35%,var(--border))] text-[var(--warning)] *:[svg]:text-current",
        error:
          "border-[color-mix(in_oklab,var(--error)_35%,var(--border))] text-[var(--error)] *:[svg]:text-current",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

export const alertTitleClassName =
  "font-heading font-medium group-has-[>svg]/alert:col-start-2 [&_a]:underline [&_a]:underline-offset-3 [&_a]:hover:text-foreground"

export const alertDescriptionClassName =
  "text-sm font-normal text-balance text-current/60 md:text-pretty [&_a]:underline [&_a]:underline-offset-3 [&_a]:hover:text-foreground [&_p:not(:last-child)]:mb-4"

export const alertActionClassName = "absolute top-2 right-2"
