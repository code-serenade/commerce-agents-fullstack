import {
  toNumberOrNull,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"
import type {
  FormDraftGenerationMode,
  ProductFormDraftRequest,
  ProductFormDraftStreamRequest,
} from "@/api"
import type { CommerceTranslate } from "../components/commerce-types"
import type { ProductsPageState } from "./use-products-page"

export function productPayload(values: SimpleFormValues) {
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

export function productDraftPayload(
  values: SimpleFormValues,
  generationMode?: FormDraftGenerationMode
): ProductFormDraftRequest {
  const payload: ProductFormDraftRequest = {
    brand: toOptionalString(values.brand),
    category: toOptionalString(values.category),
    currency: toOptionalString(values.currency) ?? "CNY",
    description: toOptionalString(values.description),
    name: toOptionalString(values.name),
    platform: toOptionalString(values.platform),
    price: toOptionalString(values.price),
    prompt: toOptionalString(values.description),
  }

  if (generationMode) {
    payload.generationMode = generationMode
  }

  return payload
}

export function productDraftStreamPayload(
  values: SimpleFormValues
): ProductFormDraftStreamRequest {
  return {
    formType: "product",
    product: productDraftPayload(values, "agent_llm"),
  }
}

export function productDraftValues(
  draft: Awaited<
    ReturnType<ProductsPageState["createProductDraft"]>
  >["draft"]
): SimpleFormValues {
  return {
    brand: draft.brand ?? "",
    category: draft.category ?? "",
    currency: draft.currency ?? "CNY",
    description: draft.description ?? "",
    externalId: draft.externalId ?? "",
    name: draft.name,
    platform: draft.platform ?? "",
    price: draft.price ?? "",
  }
}

export function productSampleValues(t: CommerceTranslate): SimpleFormValues {
  return {
    brand: t("commerce.products.samples.product.brand"),
    category: t("commerce.products.samples.product.category"),
    currency: "CNY",
    description: t("commerce.products.samples.product.description"),
    externalId: "demo-product-001",
    name: t("commerce.products.samples.product.name"),
    platform: "douyin",
    price: "69",
  }
}

export function knowledgePayload(values: SimpleFormValues) {
  return {
    confidence: toNumberOrNull(values.confidence) ?? 0.8,
    content: values.content,
    contentType: values.contentType || "text",
    metadata: {},
    sourceType: values.sourceType || "manual",
    title: toOptionalString(values.title),
  }
}
