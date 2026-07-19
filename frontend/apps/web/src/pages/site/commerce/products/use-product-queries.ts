import { useMemo } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  getProductApi,
  getProductCompetitorApi,
  getProductMaterialApi,
  listCompetitorChangesApi,
  listProductAssetsApi,
  listProductCompetitorsApi,
  listProductKnowledgeApi,
  listProductMaterialsApi,
  listProductTemplatesApi,
  listProductsApi,
} from "@/api"

export const productsQueryKey = ["commerce", "products"]
export const productTemplatesQueryKey = [...productsQueryKey, "templates"]
export const productListQueryKey = (keyword: string) => [
  ...productsQueryKey,
  "list",
  { keyword: keyword || "", status: "active" },
]

export function useProductQueries({
  keyword,
  materialStatus,
  selectedAssetId,
  selectedCompetitorId,
  selectedKnowledgeId,
  selectedMaterialId,
  selectedProductId,
}: {
  keyword: string
  materialStatus: string
  selectedAssetId: string | null
  selectedCompetitorId: string | null
  selectedKnowledgeId: string | null
  selectedMaterialId: string | null
  selectedProductId: string | null
}) {
  const productsQuery = useQuery({
    queryKey: productListQueryKey(keyword),
    queryFn: () =>
      listProductsApi({
        status: "active",
        keyword: keyword || undefined,
      }),
  })
  const productTemplatesQuery = useQuery({
    queryKey: productTemplatesQueryKey,
    queryFn: listProductTemplatesApi,
  })
  const products = useMemo(() => productsQuery.data ?? [], [productsQuery.data])
  const productTemplates = useMemo(
    () => productTemplatesQuery.data ?? [],
    [productTemplatesQuery.data]
  )
  const selectedProductSnapshot = useMemo(() => {
    if (!selectedProductId) return null

    return products.find((product) => product.id === selectedProductId) ?? null
  }, [products, selectedProductId])
  const productId = selectedProductSnapshot?.id
  const productDetailQuery = useQuery({
    queryKey: [...productsQueryKey, productId, "detail"],
    enabled: Boolean(productId),
    queryFn: () => getProductApi(productId!),
  })
  const selectedProduct = productDetailQuery.data ?? selectedProductSnapshot

  const knowledgeQuery = useQuery({
    queryKey: [...productsQueryKey, productId, "knowledge"],
    enabled: Boolean(productId),
    queryFn: () => listProductKnowledgeApi(productId!, { status: "active" }),
  })
  const knowledge = useMemo(
    () => knowledgeQuery.data ?? [],
    [knowledgeQuery.data]
  )
  const selectedKnowledge = useMemo(() => {
    if (!knowledge.length) return null

    return (
      knowledge.find((item) => item.id === selectedKnowledgeId) ??
      knowledge[0]
    )
  }, [knowledge, selectedKnowledgeId])
  const assetsQuery = useQuery({
    queryKey: [...productsQueryKey, productId, "assets"],
    enabled: Boolean(productId),
    queryFn: () => listProductAssetsApi(productId!, { status: "active" }),
  })
  const assets = useMemo(() => assetsQuery.data ?? [], [assetsQuery.data])
  const selectedAsset = useMemo(() => {
    if (!assets.length) return null

    return assets.find((item) => item.id === selectedAssetId) ?? assets[0]
  }, [assets, selectedAssetId])
  const materialsQuery = useQuery({
    queryKey: [
      ...productsQueryKey,
      productId,
      "materials",
      materialStatus || "all",
    ],
    enabled: Boolean(productId),
    queryFn: async () => {
      if (materialStatus) {
        return listProductMaterialsApi(productId!, { status: materialStatus })
      }

      const [activeMaterials, archivedMaterials] = await Promise.all([
        listProductMaterialsApi(productId!, { status: "active" }),
        listProductMaterialsApi(productId!, { status: "archived" }),
      ])
      return [...activeMaterials, ...archivedMaterials]
    },
  })
  const materials = useMemo(
    () => materialsQuery.data ?? [],
    [materialsQuery.data]
  )
  const selectedMaterialSnapshot = useMemo(() => {
    if (!materials.length) return null

    return (
      materials.find((material) => material.id === selectedMaterialId) ??
      materials[0]
    )
  }, [materials, selectedMaterialId])
  const materialDetailQuery = useQuery({
    queryKey: [
      ...productsQueryKey,
      productId,
      "materials",
      selectedMaterialSnapshot?.id,
      "detail",
    ],
    enabled: Boolean(
      productId &&
        selectedMaterialSnapshot?.id &&
        selectedMaterialSnapshot.status !== "archived"
    ),
    queryFn: () =>
      getProductMaterialApi(productId!, selectedMaterialSnapshot!.id),
  })
  const selectedMaterial =
    materialDetailQuery.data ?? selectedMaterialSnapshot
  const competitorsQuery = useQuery({
    queryKey: [...productsQueryKey, productId, "competitors"],
    enabled: Boolean(productId),
    queryFn: () => listProductCompetitorsApi(productId!, { status: "active" }),
  })
  const competitors = useMemo(
    () => competitorsQuery.data ?? [],
    [competitorsQuery.data]
  )
  const selectedCompetitorSnapshot = useMemo(() => {
    if (!competitors.length) return null

    return (
      competitors.find((competitor) => competitor.id === selectedCompetitorId) ??
      competitors[0]
    )
  }, [competitors, selectedCompetitorId])
  const competitorDetailQuery = useQuery({
    queryKey: [
      ...productsQueryKey,
      productId,
      "competitors",
      selectedCompetitorSnapshot?.id,
      "detail",
    ],
    enabled: Boolean(productId && selectedCompetitorSnapshot?.id),
    queryFn: () =>
      getProductCompetitorApi(productId!, selectedCompetitorSnapshot!.id),
  })
  const selectedCompetitor =
    competitorDetailQuery.data ?? selectedCompetitorSnapshot
  const changesQuery = useQuery({
    queryKey: [
      ...productsQueryKey,
      productId,
      "competitors",
      selectedCompetitor?.id,
      "changes",
    ],
    enabled: Boolean(productId && selectedCompetitor?.id),
    queryFn: () =>
      listCompetitorChangesApi(productId!, selectedCompetitor!.id, {
        limit: 20,
      }),
  })

  return {
    assets,
    changes: changesQuery.data ?? [],
    competitors,
    knowledge,
    loading:
      productsQuery.isLoading ||
      productDetailQuery.isLoading ||
      knowledgeQuery.isLoading ||
      assetsQuery.isLoading ||
      materialsQuery.isLoading ||
      materialDetailQuery.isLoading ||
      competitorsQuery.isLoading ||
      competitorDetailQuery.isLoading,
    loadingProductTemplates: productTemplatesQuery.isLoading,
    materials,
    productTemplates,
    products,
    selectedAsset,
    selectedCompetitor,
    selectedKnowledge,
    selectedMaterial,
    selectedProduct,
  }
}
