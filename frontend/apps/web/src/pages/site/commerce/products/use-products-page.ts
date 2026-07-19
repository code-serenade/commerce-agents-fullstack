import { useState } from "react"
import { useTranslation } from "react-i18next"
import { useProductActions } from "./use-product-actions"
import { useProductQueries } from "./use-product-queries"

export function useProductsPage() {
  const { t } = useTranslation()
  const [keyword, setKeyword] = useState("")
  const [detailOpen, setDetailOpen] = useState(false)
  const [selectedProductId, setSelectedProductId] = useState<string | null>(null)
  const [selectedKnowledgeId, setSelectedKnowledgeId] = useState<string | null>(null)
  const [selectedAssetId, setSelectedAssetId] = useState<string | null>(null)
  const [selectedCompetitorId, setSelectedCompetitorId] = useState<string | null>(null)
  const [selectedMaterialId, setSelectedMaterialId] = useState<string | null>(null)
  const [materialStatus, setMaterialStatus] = useState("active")
  const [dialog, setDialog] = useState<ProductDialogName | null>(null)
  const [assetFile, setAssetFile] = useState<File | null>(null)

  const queries = useProductQueries({
    keyword,
    materialStatus,
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
    setDetailOpen,
    t,
  })

  return {
    ...actions,
    ...queries,
    assetFile,
    detailOpen,
    dialog,
    keyword,
    materialStatus,
    openProductEditor: (productId: string) => {
      setSelectedProductId(productId)
      setDialog("edit-product")
    },
    openProductDetail: (productId: string) => {
      setSelectedProductId(productId)
      setDetailOpen(true)
    },
    setAssetFile,
    setDetailOpen,
    setDialog,
    setKeyword,
    setMaterialStatus,
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
