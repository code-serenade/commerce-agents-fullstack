import { useMutation, useQueryClient } from "@tanstack/react-query"
import {
  accessProductAssetApi,
  archiveProductAssetApi,
  archiveProductKnowledgeApi,
  archiveProductMaterialApi,
  archiveProductApi,
  createProductApi,
  createProductKnowledgeApi,
  createProductMaterialApi,
  updateProductAssetApi,
  updateProductKnowledgeApi,
  updateProductMaterialApi,
  updateProductApi,
  type ProductAssetResponse,
  type ProductKnowledgeResponse,
  type ProductMaterialResponse,
  type ProductResponse,
} from "@/api"
import {
  notifySuccess,
  toNumberOrNull,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"
import type { CommerceTranslate } from "../components/commerce-types"
import type { ProductDialogName } from "./use-products-page"
import { productsQueryKey } from "./use-product-queries"
import { createAssetRecord } from "./product-asset-action"
import { useProductCompetitorActions } from "./use-product-competitor-actions"
import { syncProductCache } from "./product-cache"

export function useProductActions({
  assetFile,
  productId,
  selectedCompetitorId,
  setAssetFile,
  setDialog,
  setSelectedAssetId,
  setSelectedCompetitorId,
  setSelectedKnowledgeId,
  setSelectedMaterialId,
  setSelectedProductId,
  t,
}: {
  assetFile: File | null
  productId: string | undefined
  selectedCompetitorId: string | undefined
  setAssetFile: (file: File | null) => void
  setDialog: (dialog: ProductDialogName | null) => void
  setSelectedAssetId: (id: string | null) => void
  setSelectedCompetitorId: (id: string | null) => void
  setSelectedKnowledgeId: (id: string | null) => void
  setSelectedMaterialId: (id: string | null) => void
  setSelectedProductId: (id: string | null) => void
  t: CommerceTranslate
}) {
  const queryClient = useQueryClient()
  const invalidateProducts = async () => {
    await queryClient.invalidateQueries({ queryKey: productsQueryKey })
  }
  const invalidateSelectedProduct = async () => {
    await queryClient.invalidateQueries({
      queryKey: [...productsQueryKey, productId],
    })
    await invalidateProducts()
  }

  const createProduct = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createProductApi(productPayload(values)),
    onSuccess: async (product) => {
      syncProductCache(queryClient, product)
      setSelectedProductId(product.id)
      setDialog(null)
      notifySuccess(t("commerce.products.notify.created"))
      await invalidateProducts()
    },
  })
  const updateProduct = useMutation({
    mutationFn: ({
      target,
      values,
    }: {
      target: ProductResponse
      values: SimpleFormValues
    }) =>
      updateProductApi(target.id, {
        ...productPayload(values),
        metadata: target.metadata ?? {},
      }),
    onSuccess: async (product) => {
      syncProductCache(queryClient, product)
      setDialog(null)
      notifySuccess(t("commerce.products.notify.updated"))
      await invalidateProducts()
    },
  })
  const archiveProduct = useMutation({
    mutationFn: archiveProductApi,
    onSuccess: async () => {
      setSelectedProductId(null)
      notifySuccess(t("commerce.products.notify.archived"))
      await invalidateProducts()
    },
  })
  const createKnowledge = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createProductKnowledgeApi(productId!, knowledgePayload(values)),
    onSuccess: async (knowledge) => {
      setSelectedKnowledgeId(knowledge.id)
      setDialog(null)
      notifySuccess(t("commerce.products.notify.knowledgeCreated"))
      await invalidateSelectedProduct()
    },
  })
  const updateKnowledge = useMutation({
    mutationFn: ({
      target,
      values,
    }: {
      target: ProductKnowledgeResponse
      values: SimpleFormValues
    }) =>
      updateProductKnowledgeApi(productId!, target.id, {
        ...knowledgePayload(values),
        metadata: target.metadata ?? {},
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.products.notify.knowledgeUpdated"))
      await invalidateSelectedProduct()
    },
  })
  const archiveKnowledge = useMutation({
    mutationFn: (knowledgeId: string) =>
      archiveProductKnowledgeApi(productId!, knowledgeId),
    onSuccess: async () => {
      setSelectedKnowledgeId(null)
      notifySuccess(t("commerce.products.notify.knowledgeArchived"))
      await invalidateSelectedProduct()
    },
  })
  const createMaterial = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createProductMaterialApi(productId!, {
        materialType: values.materialType,
        title: toOptionalString(values.title),
        content: toOptionalString(values.content),
        metadata: {},
      }),
    onSuccess: async (material) => {
      setSelectedMaterialId(material.id)
      setDialog(null)
      notifySuccess(t("commerce.products.notify.materialCreated"))
      await invalidateSelectedProduct()
    },
  })
  const updateMaterial = useMutation({
    mutationFn: ({
      target,
      values,
    }: {
      target: ProductMaterialResponse
      values: SimpleFormValues
    }) =>
      updateProductMaterialApi(productId!, target.id, {
        content: toOptionalString(values.content),
        metadata: target.metadata ?? {},
        title: toOptionalString(values.title),
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.products.notify.materialUpdated"))
      await invalidateSelectedProduct()
    },
  })
  const archiveMaterial = useMutation({
    mutationFn: (materialId: string) =>
      archiveProductMaterialApi(productId!, materialId),
    onSuccess: async () => {
      setSelectedMaterialId(null)
      notifySuccess(t("commerce.products.notify.materialArchived"))
      await invalidateSelectedProduct()
    },
  })
  const createAsset = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createAssetRecord({ file: assetFile, productId: productId!, values }),
    onSuccess: async (asset) => {
      setSelectedAssetId(asset.id)
      setAssetFile(null)
      setDialog(null)
      notifySuccess(t("commerce.products.notify.assetCreated"))
      await invalidateSelectedProduct()
    },
  })
  const updateAsset = useMutation({
    mutationFn: ({
      target,
      values,
    }: {
      target: ProductAssetResponse
      values: SimpleFormValues
    }) =>
      updateProductAssetApi(productId!, target.id, {
        assetType: values.assetType || target.assetType,
        description: toOptionalString(values.description),
        fileName: toOptionalString(values.fileName),
        fileSize: toNumberOrNull(values.fileSize),
        metadata: target.metadata ?? {},
        mimeType: toOptionalString(values.mimeType),
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.products.notify.assetUpdated"))
      await invalidateSelectedProduct()
    },
  })
  const archiveAsset = useMutation({
    mutationFn: (assetId: string) => archiveProductAssetApi(productId!, assetId),
    onSuccess: async () => {
      setSelectedAssetId(null)
      notifySuccess(t("commerce.products.notify.assetArchived"))
      await invalidateSelectedProduct()
    },
  })
  const accessAsset = useMutation({
    mutationFn: (assetId: string) => accessProductAssetApi(productId!, assetId),
    onSuccess: (result) => {
      window.open(result.downloadUrl, "_blank", "noopener,noreferrer")
    },
  })
  const {
    archiveCompetitor,
    createCompetitor,
    createCompetitorChange,
    updateCompetitor,
  } = useProductCompetitorActions({
    invalidateSelectedProduct,
    productId,
    selectedCompetitorId,
    setDialog,
    setSelectedCompetitorId,
    t,
  })

  return {
    accessAsset: accessAsset.mutate,
    archiveAsset: archiveAsset.mutate,
    archiveCompetitor,
    archiveKnowledge: archiveKnowledge.mutate,
    archiveMaterial: archiveMaterial.mutate,
    archiveProduct: archiveProduct.mutate,
    createAsset: createAsset.mutate,
    createCompetitor,
    createCompetitorChange,
    createKnowledge: createKnowledge.mutate,
    createMaterial: createMaterial.mutate,
    createProduct: createProduct.mutate,
    updateAsset: updateAsset.mutate,
    updateCompetitor,
    updateKnowledge: updateKnowledge.mutate,
    updateMaterial: updateMaterial.mutate,
    updateProduct: updateProduct.mutate,
  }
}

function productPayload(values: SimpleFormValues) {
  return {
    name: values.name,
    platform: toOptionalString(values.platform),
    externalId: toOptionalString(values.externalId),
    category: toOptionalString(values.category),
    brand: toOptionalString(values.brand),
    description: toOptionalString(values.description),
    price: toOptionalString(values.price),
    currency: values.currency || "CNY",
    metadata: {},
  }
}

function knowledgePayload(values: SimpleFormValues) {
  return {
    confidence: toNumberOrNull(values.confidence) ?? 0.8,
    content: values.content,
    contentType: values.contentType || "text",
    metadata: {},
    sourceType: values.sourceType || "manual",
    title: toOptionalString(values.title),
  }
}
