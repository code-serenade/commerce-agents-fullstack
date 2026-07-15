import type { ReactNode } from "react"

export const minimalAuthPanelClassName =
  "relative z-10 mx-auto mt-[-10px] w-full max-w-[420px]"

export const minimalAuthBrandClassName =
  "auth-login-brand-title m-0 text-[32px] leading-tight font-bold tracking-normal text-(--auth-minimal-heading-foreground) drop-shadow-[var(--auth-minimal-brand-shadow)]"

export const minimalAuthSubtitleClassName =
  "mx-auto my-0 max-w-[360px] text-[15px] leading-[1.8] text-(--auth-minimal-muted-foreground)"

export const minimalAuthLabelClassName =
  "mb-[9px] block text-sm font-bold text-(--auth-minimal-label-foreground)"

export const minimalAuthInputClassName =
  "h-12 w-full rounded-xl border border-(--auth-minimal-field-border) [background:var(--auth-minimal-field-background)] px-3.5 text-[15px] text-(--auth-minimal-field-foreground) shadow-[var(--auth-minimal-field-shadow)] transition outline-none focus:border-(--auth-minimal-field-focus-border) focus:shadow-[var(--auth-minimal-field-focus-shadow)]"

export const minimalAuthPasswordInputClassName = `${minimalAuthInputClassName} pr-12`

export const minimalAuthIconButtonClassName =
  "absolute top-1/2 right-3 inline-flex size-8 -translate-y-1/2 cursor-pointer items-center justify-center rounded-lg text-(--auth-minimal-muted-foreground) transition hover:bg-(--auth-minimal-icon-hover-background) hover:text-(--auth-minimal-action-foreground) focus-visible:ring-2 focus-visible:ring-ring/25 focus-visible:outline-none"

export const minimalAuthSubmitClassName =
  "mt-[26px] h-12 w-full cursor-pointer rounded-xl border-0 bg-[image:var(--auth-minimal-submit-background)] text-base font-bold text-(--auth-minimal-submit-foreground) shadow-[var(--auth-minimal-submit-shadow)] transition disabled:cursor-not-allowed disabled:opacity-65"

export const minimalAuthFooterClassName =
  "mt-6 flex justify-center gap-2 text-sm text-(--auth-minimal-muted-foreground)"

export const minimalAuthFooterLinkClassName =
  "cursor-pointer border-0 bg-transparent p-0 font-bold text-(--auth-minimal-action-foreground) underline-offset-4 transition hover:underline"

export interface MinimalAuthLayoutProps {
  brand: string
  brandLogo?: ReactNode
  children: ReactNode
  footer?: ReactNode
  subtitle: string
}

export function MinimalAuthLayout({
  brand,
  brandLogo,
  children,
  footer,
  subtitle,
}: MinimalAuthLayoutProps) {
  return (
    <section className={minimalAuthPanelClassName}>
      <div className="mb-[18px] flex flex-col items-center justify-center gap-3 text-center">
        {brandLogo ? <div className="shrink-0">{brandLogo}</div> : null}
        <p className={minimalAuthBrandClassName}>{brand}</p>
      </div>

      <header className="mb-[34px] text-center">
        <p className={minimalAuthSubtitleClassName}>{subtitle}</p>
      </header>

      {children}

      {footer ? <p className={minimalAuthFooterClassName}>{footer}</p> : null}
    </section>
  )
}
