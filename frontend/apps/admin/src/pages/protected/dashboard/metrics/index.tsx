import { MetricCards, type MetricCardsItem } from "@workspace/app-kit"

export function DashboardMetrics({ items }: { items: MetricCardsItem[] }) {
  return <MetricCards items={items} />
}
