import { MetricCards } from "@workspace/app-kit"
import type { MetricCardsItem } from "@workspace/app-kit"

export function ProductsMetrics({ items }: { items: MetricCardsItem[] }) {
  return <MetricCards items={items} />
}
