import {
  normalizeNumericInput,
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { ProductsPageState } from "../use-products-page"

export function productInitialValues(
  page: ProductsPageState
): SimpleFormValues {
  return {
    brand: page.selectedProduct?.brand ?? "",
    category: page.selectedProduct?.category ?? "",
    currency: page.selectedProduct?.currency ?? "CNY",
    description: page.selectedProduct?.description ?? "",
    externalId: page.selectedProduct?.externalId ?? "",
    name: page.selectedProduct?.name ?? "",
    platform: page.selectedProduct?.platform ?? "",
    price: normalizeNumericInput(page.selectedProduct?.price ?? "", {
      decimalScale: 2,
      nonNegative: true,
    }),
  }
}

export function productFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "name", label: t("commerce.fields.productName"), required: true },
    {
      key: "platform",
      label: t("commerce.common.platform"),
      placeholder: String("douyin / taobao / jd"),
    },
    { key: "externalId", label: t("commerce.fields.platformProductId") },
    { key: "category", label: t("commerce.fields.category") },
    { key: "brand", label: t("commerce.fields.brand") },
    {
      key: "currency",
      label: t("commerce.fields.currency"),
      layout: "half",
    },
    {
      decimalScale: 2,
      key: "price",
      label: t("commerce.fields.price"),
      layout: "half",
      nonNegative: true,
      type: "number",
    },
    {
      key: "description",
      label: t("commerce.fields.description"),
      type: "textarea",
    },
  ]
}

export function knowledgeFields(
  t: CommerceTranslate,
  { includeSourceType = true }: { includeSourceType?: boolean } = {}
): SimpleField[] {
  const fields: SimpleField[] = [
    { key: "title", label: t("commerce.fields.title") },
    {
      key: "contentType",
      label: t("commerce.fields.contentType"),
      type: "select",
      options: knowledgeContentTypeOptions(t),
    },
    {
      key: "confidence",
      label: t("commerce.products.knowledgeConfidence.label"),
      type: "select",
      options: knowledgeConfidenceOptions(t),
    },
    {
      key: "content",
      label: t("commerce.fields.content"),
      required: true,
      type: "textarea",
    },
  ]

  if (includeSourceType) {
    fields.splice(2, 0, {
      key: "sourceType",
      label: t("commerce.fields.sourceType"),
      type: "select",
      options: knowledgeSourceTypeOptions(t),
    })
  }

  return fields
}

export function knowledgeContentTypeOptions(t: CommerceTranslate) {
  return [
    {
      label: t("commerce.products.knowledgeContentTypes.sellingPoint"),
      value: "selling_point",
    },
    { label: t("commerce.products.knowledgeContentTypes.text"), value: "text" },
    { label: t("commerce.products.knowledgeContentTypes.spec"), value: "spec" },
    { label: t("commerce.products.knowledgeContentTypes.faq"), value: "faq" },
    {
      label: t("commerce.products.knowledgeContentTypes.competitive"),
      value: "competitive",
    },
  ]
}

export function knowledgeSourceTypeOptions(t: CommerceTranslate) {
  return [
    { label: t("commerce.products.knowledgeSourceTypes.manual"), value: "manual" },
    {
      label: t("commerce.products.knowledgeSourceTypes.agentGenerated"),
      value: "agent_generated",
    },
    {
      label: t("commerce.products.knowledgeSourceTypes.platformSync"),
      value: "platform_sync",
    },
    {
      label: t("commerce.products.knowledgeSourceTypes.imported"),
      value: "imported",
    },
  ]
}

export function knowledgeConfidenceOptions(t: CommerceTranslate) {
  return [
    { label: t("commerce.products.knowledgeConfidence.low"), value: "0.5" },
    { label: t("commerce.products.knowledgeConfidence.medium"), value: "0.8" },
    { label: t("commerce.products.knowledgeConfidence.high"), value: "0.95" },
  ]
}

export function knowledgeContentTypeLabel(
  t: CommerceTranslate,
  contentType: string
) {
  return (
    knowledgeContentTypeOptions(t).find((option) => option.value === contentType)
      ?.label ?? contentType
  )
}

export function knowledgeConfidenceLabel(
  t: CommerceTranslate,
  confidence?: number | null
) {
  if (confidence == null) return "-"
  const normalizedConfidence = String(confidence)

  return (
    knowledgeConfidenceOptions(t).find(
      (option) => Number(option.value) === confidence
    )?.label ?? normalizedConfidence
  )
}

export function knowledgeSourceTypeLabel(
  t: CommerceTranslate,
  sourceType?: string | null
) {
  if (!sourceType) return t("commerce.products.knowledgeSourceTypes.manual")
  return (
    knowledgeSourceTypeOptions(t).find((option) => option.value === sourceType)
      ?.label ?? sourceType
  )
}

export function knowledgeInitialValues(
  page: ProductsPageState
): SimpleFormValues {
  return {
    confidence: String(page.selectedKnowledge?.confidence ?? "0.8"),
    content: page.selectedKnowledge?.content ?? "",
    contentType: page.selectedKnowledge?.contentType ?? "text",
    sourceType: page.selectedKnowledge?.sourceType ?? "manual",
    title: page.selectedKnowledge?.title ?? "",
  }
}

export function assetFields(t: CommerceTranslate): SimpleField[] {
  return [
    {
      key: "assetType",
      label: t("commerce.fields.assetType"),
      placeholder: String("docs / images / videos / other"),
    },
    { key: "fileName", label: t("commerce.fields.fileName") },
    { key: "objectKey", label: t("commerce.fields.objectKey") },
    { key: "mimeType", label: t("commerce.fields.mimeType") },
    { key: "fileSize", label: t("commerce.fields.fileSize") },
    { key: "description", label: t("commerce.fields.note") },
  ]
}

export function assetInitialValues(page: ProductsPageState): SimpleFormValues {
  return {
    assetType: page.selectedAsset?.assetType ?? "docs",
    description: page.selectedAsset?.description ?? "",
    fileName: page.selectedAsset?.fileName ?? "",
    fileSize: String(page.selectedAsset?.fileSize ?? ""),
    mimeType: page.selectedAsset?.mimeType ?? "",
    objectKey: page.selectedAsset?.objectKey ?? "",
  }
}

export function materialFields(t: CommerceTranslate): SimpleField[] {
  return [
    {
      key: "materialType",
      label: t("commerce.fields.materialType"),
      required: true,
    },
    { key: "title", label: t("commerce.fields.title") },
    { key: "content", label: t("commerce.fields.content"), type: "textarea" },
  ]
}

export function materialInitialValues(
  page: ProductsPageState
): SimpleFormValues {
  return {
    content: page.selectedMaterial?.content ?? "",
    materialType: page.selectedMaterial?.materialType ?? "copy",
    title: page.selectedMaterial?.title ?? "",
  }
}

export function competitorFields(t: CommerceTranslate): SimpleField[] {
  return [
    {
      key: "name",
      label: t("commerce.fields.competitorName"),
      required: true,
    },
    { key: "platform", label: t("commerce.common.platform") },
    { key: "url", label: t("commerce.fields.url") },
    { key: "price", label: t("commerce.fields.price"), type: "number" },
    { key: "rating", label: t("commerce.fields.rating"), type: "number" },
    { key: "monthlySales", label: t("commerce.fields.monthlySales") },
    { key: "notes", label: t("commerce.fields.notes"), type: "textarea" },
  ]
}

export function competitorInitialValues(
  page: ProductsPageState
): SimpleFormValues {
  return {
    monthlySales: page.selectedCompetitor?.monthlySales ?? "",
    name: page.selectedCompetitor?.name ?? "",
    notes: page.selectedCompetitor?.notes ?? "",
    platform: page.selectedCompetitor?.platform ?? "",
    price: String(page.selectedCompetitor?.price ?? ""),
    rating: String(page.selectedCompetitor?.rating ?? ""),
    url: page.selectedCompetitor?.url ?? "",
  }
}

export function changeFields(t: CommerceTranslate): SimpleField[] {
  return [
    {
      key: "changeType",
      label: t("commerce.fields.changeType"),
      required: true,
    },
    { key: "oldValue", label: t("commerce.fields.oldValue") },
    { key: "newValue", label: t("commerce.fields.newValue") },
    { key: "sourceType", label: t("commerce.fields.sourceType") },
    { key: "summary", label: t("commerce.fields.summary"), type: "textarea" },
  ]
}
