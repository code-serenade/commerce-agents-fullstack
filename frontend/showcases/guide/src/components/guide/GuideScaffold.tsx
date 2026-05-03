import type { ReactNode } from "react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@workspace/ui-components"

interface GuidePageProps {
  eyebrow?: ReactNode
  title: ReactNode
  description: ReactNode
  badges?: ReactNode[]
  stats?: Array<{ label: ReactNode; value: ReactNode }>
  children: ReactNode
}

interface GuideSectionProps {
  id?: string
  title: ReactNode
  description?: ReactNode
  children: ReactNode
}

interface DemoBlockProps {
  title: ReactNode
  description?: ReactNode
  children: ReactNode
}

export function GuidePage({
  eyebrow = "UI Components",
  title,
  description,
  badges,
  stats,
  children,
}: GuidePageProps) {
  return (
    <div className="min-h-0 flex-1 overflow-auto">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-6 p-4 md:p-6">
        <section className="overflow-hidden rounded-[28px] border border-border/60 bg-[linear-gradient(135deg,color-mix(in_oklab,var(--primary)_14%,var(--card))_0%,var(--card)_44%,color-mix(in_oklab,var(--accent)_18%,var(--card))_100%)]">
          <div className="flex flex-col gap-6 p-6 md:p-8">
            <div className="space-y-3">
              <p className="text-xs font-medium uppercase tracking-[0.24em] text-muted-foreground">
                {eyebrow}
              </p>
              <h1 className="text-3xl font-semibold tracking-tight md:text-4xl">
                {title}
              </h1>
              <p className="max-w-3xl text-sm/6  text-muted-foreground md:text-base">
                {description}
              </p>
            </div>

            {badges?.length ? (
              <div className="flex flex-wrap items-center gap-3">{badges}</div>
            ) : null}

            {stats?.length ? (
              <div className="grid gap-3 md:grid-cols-3">
                {stats.map((stat) => (
                  <div
                    key={String(stat.label)}
                    className="rounded-2xl border border-border/60 bg-card/80 p-4"
                  >
                    <p className="text-xs uppercase tracking-[0.2em] text-muted-foreground">
                      {stat.label}
                    </p>
                    <p className="mt-2 text-2xl font-semibold tracking-tight">
                      {stat.value}
                    </p>
                  </div>
                ))}
              </div>
            ) : null}
          </div>
        </section>

        {children}
      </div>
    </div>
  )
}

export function GuideSection({
  id,
  title,
  description,
  children,
}: GuideSectionProps) {
  return (
    <section id={id}>
      <Card>
        <CardHeader>
          <CardTitle>{title}</CardTitle>
          {description ? <CardDescription>{description}</CardDescription> : null}
        </CardHeader>
        <CardContent>
          <div className="space-y-4">{children}</div>
        </CardContent>
      </Card>
    </section>
  )
}

export function DemoGrid({ children }: { children: ReactNode }) {
  return <div className="grid gap-4 lg:grid-cols-2">{children}</div>
}

export function DemoBlock({
  title,
  description,
  children,
}: DemoBlockProps) {
  return (
    <div className="rounded-2xl border border-border/60 bg-card/70 p-4">
      <div className="mb-4 space-y-1">
        <p className="text-sm font-medium">{title}</p>
        {description ? (
          <p className="text-xs/5  text-muted-foreground">{description}</p>
        ) : null}
      </div>
      {children}
    </div>
  )
}
