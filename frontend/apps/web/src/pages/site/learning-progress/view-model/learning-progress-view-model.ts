import type { LearningEvidenceEntry, LearningProfileResponse } from "@/api"
import {
  getAssessedKnowledgeCount,
  getDomainEvaluationStatus,
} from "../learning-progress-evaluation"
import type { LearningProgressViewModel } from "../types"
import {
  buildLearningCapabilityRows,
  buildLearningPortfolioDays,
  filterLearningPortfolioDays,
} from "../domain/learning-support-surfaces"

export const learningDomainCodes = [
  "market",
  "product",
  "content",
  "traffic",
  "conversion",
  "finance",
  "service",
  "compliance",
] as const

export function buildLearningProgressViewModel(
  profile: LearningProfileResponse | undefined,
  evidence: LearningEvidenceEntry[] | undefined
): LearningProgressViewModel {
  const domainAssessments = profile?.domainAssessments ?? []
  const portfolioDays = buildLearningPortfolioDays(evidence, domainAssessments)
  const portfolioItems = portfolioDays.flatMap((day) => day.items)
  const correctCount = portfolioItems.filter(
    (item) => item.outcome === "correct"
  ).length
  const rawDomains = learningDomainCodes.map((code) => ({
    code,
    observed: Boolean(profile?.masteryMap?.[code]),
    evaluationStatus: getDomainEvaluationStatus(
      profile?.masteryMap?.[code],
      evidence,
      code
    ),
    assessment: domainAssessments.find(
      (assessment) => assessment.knowledgeCode === code
    ),
  }))

  return {
    assessedCount:
      profile?.assessmentSummary?.assessedKnowledgeCount ??
      getAssessedKnowledgeCount(profile?.masteryMap),
    coverageRate: Math.round((profile?.coverageRate ?? 0) * 100),
    demonstratedCount:
      profile?.assessmentSummary?.demonstratedKnowledgeCount ?? 0,
    evidenceCount: Number(profile?.engagementSummary?.evidenceCount ?? 0),
    analyzedMessages: Number(
      profile?.engagementSummary?.analyzedMessages ?? 0
    ),
    multiAngleCount:
      profile?.assessmentSummary?.multiAngleKnowledgeCount ?? 0,
    observedCount: Object.keys(profile?.masteryMap ?? {}).length,
    retainedCount: profile?.assessmentSummary?.retainedKnowledgeCount ?? 0,
    domainAssessments,
    capabilityRows: buildLearningCapabilityRows(
      learningDomainCodes,
      domainAssessments
    ),
    portfolioDays,
    reviewDays: filterLearningPortfolioDays(portfolioDays),
    portfolioSummary: {
      total: portfolioItems.length,
      correct: correctCount,
      review: portfolioItems.length - correctCount,
    },
    rawDomains,
  }
}
