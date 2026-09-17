import type { LearningEvidenceEntry } from "@/api"

export function getAssessedKnowledgeCount(
  masteryMap: Record<string, unknown> | undefined
) {
  return Object.values(masteryMap ?? {}).filter((value) => {
    if (!value || typeof value !== "object" || Array.isArray(value)) {
      return false
    }
    const assessedCount = (value as Record<string, unknown>).assessedCount
    return typeof assessedCount === "number" && assessedCount > 0
  }).length
}

export function getDomainEvaluationStatus(
  masteryEntry: unknown,
  evidence: LearningEvidenceEntry[] | undefined,
  knowledgeCode: string
) {
  const outcome = evidence?.find(
    (item) =>
      item.knowledgeCode === knowledgeCode &&
      isDeterministicAssessment(item.assessmentOutcome)
  )?.assessmentOutcome
  if (outcome === "correct") return "assessed"
  if (outcome === "partial") return "developing"
  if (outcome === "incorrect") return "needs_attention"
  return getProfileEvaluationStatus(masteryEntry)
}

function isDeterministicAssessment(outcome: string | null | undefined) {
  return outcome === "correct" || outcome === "partial" || outcome === "incorrect"
}

function getProfileEvaluationStatus(value: unknown) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return undefined
  }
  const status = (value as Record<string, unknown>).evaluationStatus
  return typeof status === "string" ? status : undefined
}
