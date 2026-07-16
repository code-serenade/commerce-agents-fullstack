import type { CommerceTranslate } from "../components/commerce-types"

const knowledgeContentTypes = [
  ["selling_point", "commerce.products.knowledgeContentTypes.sellingPoint"],
  ["text", "commerce.products.knowledgeContentTypes.text"],
  ["spec", "commerce.products.knowledgeContentTypes.spec"],
  ["faq", "commerce.products.knowledgeContentTypes.faq"],
  ["competitive", "commerce.products.knowledgeContentTypes.competitive"],
] as const

const knowledgeSourceTypes = [
  ["manual", "commerce.products.knowledgeSourceTypes.manual"],
  ["agent_generated", "commerce.products.knowledgeSourceTypes.agentGenerated"],
  ["platform_sync", "commerce.products.knowledgeSourceTypes.platformSync"],
  ["imported", "commerce.products.knowledgeSourceTypes.imported"],
] as const

const knowledgeConfidences = [
  ["0.5", "commerce.products.knowledgeConfidence.low"],
  ["0.8", "commerce.products.knowledgeConfidence.medium"],
  ["0.95", "commerce.products.knowledgeConfidence.high"],
] as const

function translatedOptions(
  t: CommerceTranslate,
  definitions: readonly (readonly [string, string])[]
) {
  return definitions.map(([value, labelKey]) => ({ label: t(labelKey), value }))
}

export function knowledgeContentTypeOptions(t: CommerceTranslate) {
  return translatedOptions(t, knowledgeContentTypes)
}

export function knowledgeSourceTypeOptions(t: CommerceTranslate) {
  return translatedOptions(t, knowledgeSourceTypes)
}

export function knowledgeConfidenceOptions(t: CommerceTranslate) {
  return translatedOptions(t, knowledgeConfidences)
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

  return (
    knowledgeConfidenceOptions(t).find(
      (option) => Number(option.value) === confidence
    )?.label ?? String(confidence)
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
