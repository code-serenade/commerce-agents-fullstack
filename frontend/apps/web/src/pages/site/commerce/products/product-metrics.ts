import type { MetricCardsItem } from "@workspace/app-kit"
import type {
  ProductAssetResponse,
  ProductCompetitorResponse,
  ProductDashboardResponse,
  ProductKnowledgeResponse,
  ProductMaterialResponse,
} from "@/api"
import type { CommerceTranslate } from "../components/commerce-types"

export function buildProductMetricCards(
  t: CommerceTranslate,
  data: {
    assets: ProductAssetResponse[]
    competitors: ProductCompetitorResponse[]
    dashboard: ProductDashboardResponse | undefined
    knowledge: ProductKnowledgeResponse[]
    materials: ProductMaterialResponse[]
  }
): MetricCardsItem[] {
  return [
    {
      key: "active",
      label: t("commerce.products.metrics.active"),
      value: data.dashboard?.active ?? 0,
      tail: t("commerce.products.metrics.activeTail", {
        archived: data.dashboard?.archived ?? 0,
        total: data.dashboard?.total ?? 0,
      }),
      variant: "accent",
    },
    {
      key: "knowledge",
      label: t("commerce.products.metrics.knowledge"),
      value: data.knowledge.length,
      tail: t("commerce.products.metrics.knowledgeTail"),
      variant: "success",
    },
    {
      key: "materials",
      label: t("commerce.products.metrics.materials"),
      value: data.assets.length + data.materials.length,
      tail: t("commerce.products.metrics.materialsTail", {
        assets: data.assets.length,
        materials: data.materials.length,
      }),
      variant: "default",
    },
    {
      key: "competitors",
      label: t("commerce.products.metrics.competitors"),
      value: data.competitors.length,
      tail: t("commerce.products.metrics.competitorsTail"),
      variant: "warning",
    },
  ]
}
