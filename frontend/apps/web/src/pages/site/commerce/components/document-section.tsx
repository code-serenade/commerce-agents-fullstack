import type { ReactNode } from "react"

function cx(...inputs: Array<string | false | null | undefined>) {
  return inputs.filter(Boolean).join(" ")
}

export function DocumentSection({
  children,
  className,
  contentClassName,
  description,
  icon,
  title,
}: {
  children: ReactNode
  className?: string
  contentClassName?: string
  description?: ReactNode
  icon: ReactNode
  title: ReactNode
}) {
  return (
    <section className={cx("min-w-0", className)}>
      <div className="mb-4 flex min-w-0 items-start gap-2">
        <span className="flex size-7 shrink-0 items-center justify-center rounded-md bg-primary/10 p-1 text-primary">
          {icon}
        </span>
        <div className="min-w-0">
          <h2 className="text-base/6 font-semibold text-foreground">{title}</h2>
          {description ? (
            <p className="mt-1 max-w-3xl text-sm/6 text-muted-foreground">
              {description}
            </p>
          ) : null}
        </div>
      </div>
      <div className={cx("pl-9", contentClassName)}>{children}</div>
    </section>
  )
}

export function DocumentIntroCard({
  description,
  icon,
  title,
}: {
  description: ReactNode
  icon: ReactNode
  title: ReactNode
}) {
  return (
    <div className="relative overflow-hidden rounded-lg border border-(--document-intro-card-border) bg-(image:--document-intro-card-background) px-4 py-3 shadow-(--document-intro-card-shadow) backdrop-blur-md before:pointer-events-none before:absolute before:inset-y-0 before:right-0 before:w-1/3 before:bg-(image:--document-intro-card-decoration) before:bg-size-[18px_18px,260px_120px] before:bg-position-[right_center,right_center] before:opacity-80">
      <div className="relative z-10 flex min-w-0 gap-3">
        <span className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary/10 p-2 text-primary [&>svg]:size-5">
          {icon}
        </span>
        <div className="min-w-0">
          <h2 className="text-sm/5 font-semibold text-foreground">{title}</h2>
          <p className="mt-1 max-w-5xl text-sm/6 text-muted-foreground">
            {description}
          </p>
        </div>
      </div>
    </div>
  )
}

export function DocumentCard({
  children,
  className,
}: {
  children: ReactNode
  className?: string
}) {
  return (
    <div
      className={cx(
        "min-w-0 rounded-lg border border-(--document-card-border) bg-(image:--document-card-background) p-4 shadow-(--document-card-shadow) backdrop-blur-md",
        className
      )}
    >
      {children}
    </div>
  )
}
