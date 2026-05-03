import type { AgentOutputMetric } from "../types"

export interface OutputMetricsProps {
  metrics: AgentOutputMetric[]
}

export function OutputMetrics({ metrics }: OutputMetricsProps) {
  return (
    <section className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
      {metrics.map((metric) => (
        <article
          key={metric.label}
          className="rounded-lg border border-(--app-border) bg-(--app-surface) p-4"
        >
          <p className="text-sm text-(--app-muted-text)">{metric.label}</p>
          <p className="mt-2 text-2xl font-semibold text-(--app-text)">
            {metric.value}
          </p>
          <p className="mt-1 text-xs/5 text-(--app-muted-text)">
            {metric.description}
          </p>
        </article>
      ))}
    </section>
  )
}
