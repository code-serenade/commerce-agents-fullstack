import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useProductActions } from "./use-product-actions"
import { useProductQueries } from "./use-product-queries"
import { buildProductMetricCards } from "./product-metrics"

export function useProductsPage() {
  const { t } = useTranslation()
  const [keyword, setKeyword] = useState("")
  const [selectedProductId, setSelectedProductId] = useState<string | null>(null)
  const [selectedKnowledgeId, setSelectedKnowledgeId] = useState<string | null>(null)
  const [selectedAssetId, setSelectedAssetId] = useState<string | null>(null)
  const [selectedCompetitorId, setSelectedCompetitorId] = useState<string | null>(null)
  const [selectedMaterialId, setSelectedMaterialId] = useState<string | null>(null)
  const [dialog, setDialog] = useState<ProductDialogName | null>(null)
  const [assetFile, setAssetFile] = useState<File | null>(null)

  const queries = useProductQueries({
    keyword,
    selectedAssetId,
    selectedCompetitorId,
    selectedKnowledgeId,
    selectedMaterialId,
    selectedProductId,
  })
  const actions = useProductActions({
    assetFile,
    productId: queries.selectedProduct?.id,
    selectedCompetitorId: queries.selectedCompetitor?.id,
    setAssetFile,
    setDialog,
    setSelectedAssetId,
    setSelectedCompetitorId,
    setSelectedKnowledgeId,
    setSelectedMaterialId,
    setSelectedProductId,
    t,
  })

  const metricCards = useMemo(
    () =>
      buildProductMetricCards(t, {
        assets: queries.assets,
        competitors: queries.competitors,
        dashboard: queries.dashboard,
        knowledge: queries.knowledge,
        materials: queries.materials,
      }),
    [
      queries.assets,
      queries.competitors,
      queries.dashboard,
      queries.knowledge,
      queries.materials,
      t,
    ]
  )

  return {
    ...actions,
    ...queries,
    dialog,
    keyword,
    metricCards,
    setAssetFile,
    setDialog,
    setKeyword,
    setSelectedAssetId,
    setSelectedCompetitorId,
    setSelectedKnowledgeId,
    setSelectedMaterialId,
    setSelectedProductId,
  }
}

export type ProductDialogName =
  | "create-product"
  | "edit-product"
  | "create-knowledge"
  | "edit-knowledge"
  | "create-asset"
  | "edit-asset"
  | "create-material"
  | "edit-material"
  | "create-competitor"
  | "edit-competitor"
  | "create-competitor-change"

export type ProductsPageState = ReturnType<typeof useProductsPage>
