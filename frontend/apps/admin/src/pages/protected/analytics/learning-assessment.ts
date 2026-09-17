import type { LearningAssessmentSummaryResponse } from "@/api"

export interface LearningAssessmentSummary {
  assessedKnowledgeCount: number
  assessmentCount: number
  distinctAssessmentQuestionCount: number
  multiAngleKnowledgeCount: number
  demonstratedKnowledgeCount: number
  retainedKnowledgeCount: number
  correctCount: number
  partialCount: number
  incorrectCount: number
  needsAttentionKnowledgeCount: number
}

export type LearningEvaluationStatus =
  | "unassessed"
  | "accumulating"
  | "demonstrated"
  | "retained"
  | "needs_attention"

function count(value: number | undefined) {
  return typeof value === "number" && Number.isFinite(value)
    ? Math.max(0, value)
    : 0
}

export function normalizeLearningAssessmentSummary(
  summary: LearningAssessmentSummaryResponse | null | undefined
): LearningAssessmentSummary {
  return {
    assessedKnowledgeCount: count(summary?.assessedKnowledgeCount),
    assessmentCount: count(summary?.assessmentCount),
    distinctAssessmentQuestionCount: count(
      summary?.distinctAssessmentQuestionCount
    ),
    multiAngleKnowledgeCount: count(summary?.multiAngleKnowledgeCount),
    demonstratedKnowledgeCount: count(summary?.demonstratedKnowledgeCount),
    retainedKnowledgeCount: count(summary?.retainedKnowledgeCount),
    correctCount: count(summary?.correctCount),
    partialCount: count(summary?.partialCount),
    incorrectCount: count(summary?.incorrectCount),
    needsAttentionKnowledgeCount: count(
      summary?.needsAttentionKnowledgeCount
    ),
  }
}

export function getLearningEvaluationStatus(
  summary: LearningAssessmentSummary
): LearningEvaluationStatus {
  if (summary.needsAttentionKnowledgeCount > 0) return "needs_attention"
  if (summary.retainedKnowledgeCount > 0) return "retained"
  if (summary.demonstratedKnowledgeCount > 0) return "demonstrated"
  if (summary.assessedKnowledgeCount > 0) return "accumulating"
  return "unassessed"
}
