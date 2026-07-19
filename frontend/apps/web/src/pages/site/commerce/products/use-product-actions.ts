import { useMutation, useQueryClient } from "@tanstack/react-query"
import {
  accessProductAssetApi,
  archiveProductAssetApi,
  archiveProductKnowledgeApi,
  archiveProductMaterialApi,
  archiveProductApi,
  createProductFormDraftApi,
  createProductApi,
  createProductKnowledgeApi,
  createProductMaterialApi,
  importProductTemplateApi,
  updateProductAssetApi,
  updateProductKnowledgeApi,
  updateProductMaterialApi,
  updateProductApi,
  type ProductAssetResponse,
  type FormDraftGenerationMode,
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
import {
  knowledgePayload,
  productDraftPayload,
  productPayload,
} from "./product-form-payloads"
import { useProductDraftStream } from "./use-product-draft-stream"

interface ProductDraftInput {
  values: SimpleFormValues
  generationMode?: FormDraftGenerationMode
}

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
  setDetailOpen,
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
  setDetailOpen: (open: boolean) => void
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
  const createProductDraft = useMutation({
    mutationFn: ({ values, generationMode }: ProductDraftInput) =>
      createProductFormDraftApi(productDraftPayload(values, generationMode)),
    onSuccess: () => {
      notifySuccess(t("commerce.products.notify.draftFilled"))
    },
  })
  const createProductDraftStream = useProductDraftStream(t)
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
  const importProductTemplate = useMutation({
    mutationFn: (templateId: string) =>
      importProductTemplateApi({ templateId }),
    onSuccess: async (result) => {
      syncProductCache(queryClient, result.product)
      setSelectedProductId(result.product.id)
      setDetailOpen(true)
      notifySuccess(
        t(
          result.created
            ? "commerce.products.notify.templateImported"
            : "commerce.products.notify.templateOpened"
        )
      )
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
    createKnowledgePending: createKnowledge.isPending,
    createMaterial: createMaterial.mutate,
    createMaterialPending: createMaterial.isPending,
    createProduct: createProduct.mutate,
    createProductDraft: createProductDraft.mutateAsync,
    createProductDraftPending:
      createProductDraft.isPending || createProductDraftStream.isPending,
    createProductDraftStream: createProductDraftStream.mutateAsync,
    createProductPending: createProduct.isPending,
    importProductTemplate: importProductTemplate.mutate,
    importProductTemplatePending: importProductTemplate.isPending,
    updateAsset: updateAsset.mutate,
    updateCompetitor,
    updateKnowledge: updateKnowledge.mutate,
    updateMaterial: updateMaterial.mutate,
    updateProduct: updateProduct.mutate,
  }
}
