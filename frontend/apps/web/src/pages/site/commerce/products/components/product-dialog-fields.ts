import {
  normalizeNumericInput,
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import {
  knowledgeConfidenceOptions,
  knowledgeContentTypeOptions,
  knowledgeSourceTypeOptions,
} from "../../domain/enum-mappings"
import type { ProductsPageState } from "../use-products-page"

export { assetFields, changeFields, materialFields } from "./product-enum-fields"

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
