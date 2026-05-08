import type { SimpleField, SimpleFormValues } from "../../components/commerce-ui"
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
    price: page.selectedProduct?.price ?? "",
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
    { key: "price", label: t("commerce.fields.price"), type: "number" },
    { key: "currency", label: t("commerce.fields.currency") },
    {
      key: "description",
      label: t("commerce.fields.description"),
      type: "textarea",
    },
  ]
}

export function knowledgeFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "title", label: t("commerce.fields.title") },
    {
      key: "contentType",
      label: t("commerce.fields.contentType"),
      required: true,
    },
    { key: "sourceType", label: t("commerce.fields.sourceType") },
    {
      key: "confidence",
      label: t("commerce.fields.confidence"),
      type: "number",
    },
    {
      key: "content",
      label: t("commerce.fields.content"),
      required: true,
      type: "textarea",
    },
  ]
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
