import type {
  LearningDomainAssessment,
  LearningEvidenceEntry,
} from "@/api"
import type {
  LearningCapabilityRow,
  LearningPortfolioDay,
  LearningPortfolioItem,
  LearningReviewItem,
} from "../types"

export type LearningPathGroup = "priority" | "evidence" | "retention"

export function buildLearningCapabilityRows(
  knowledgeCodes: readonly string[],
  assessments: LearningDomainAssessment[] | undefined
): LearningCapabilityRow[] {
  return knowledgeCodes.map((knowledgeCode) => {
    const assessment = assessments?.find(
      (item) => item.knowledgeCode === knowledgeCode
    )
    return {
      knowledgeCode,
      statusCode: assessment?.statusCode ?? "not_assessed",
      assessmentCount: assessment?.assessmentCount ?? 0,
      distinctQuestionCount: assessment?.distinctQuestionCount ?? 0,
      assessmentAngleCount: assessment?.assessmentAngleCount ?? 0,
      correctCount: assessment?.correctCount ?? 0,
      partialCount: assessment?.partialCount ?? 0,
      incorrectCount: assessment?.incorrectCount ?? 0,
      latestAssessedAt: assessment?.latestAssessedAt,
    }
  })
}

export function buildLearningPortfolioDays(
  evidence: LearningEvidenceEntry[] | undefined,
  assessments: LearningDomainAssessment[] | undefined
): LearningPortfolioDay[] {
  const items = new Map<string, LearningPortfolioItem>()

  for (const entry of evidence ?? []) {
    if (!isPortfolioOutcome(entry.assessmentOutcome)) continue
    if (!Number.isFinite(Date.parse(entry.createdAt))) continue
    items.set(entry.id, {
      id: entry.id,
      conversationId: entry.conversationId,
      knowledgeCode: entry.knowledgeCode,
      outcome: entry.assessmentOutcome,
      questionText: entry.questionText,
      selectedChoiceText: entry.selectedChoiceText,
      createdAt: entry.createdAt,
    })
  }

  for (const assessment of evidence === undefined ? assessments ?? [] : []) {
    for (const entry of assessment.recentEvidence) {
      if (!isPortfolioOutcome(entry.outcome) || items.has(entry.evidenceId)) {
        continue
      }
      if (!Number.isFinite(Date.parse(entry.createdAt))) continue
      items.set(entry.evidenceId, {
        id: entry.evidenceId,
        conversationId: entry.conversationId,
        knowledgeCode: assessment.knowledgeCode,
        outcome: entry.outcome,
        questionText: entry.questionText,
        selectedChoiceText: entry.selectedChoiceText,
        createdAt: entry.createdAt,
      })
    }
  }

  const days = new Map<string, LearningPortfolioItem[]>()
  for (const item of [...items.values()].sort(
    (left, right) => Date.parse(right.createdAt) - Date.parse(left.createdAt)
  )) {
    const date = item.createdAt.slice(0, 10)
    days.set(date, [...(days.get(date) ?? []), item])
  }

  return [...days.entries()].map(([date, dayItems]) => ({
    date,
    items: dayItems,
  }))
}

export function filterLearningPortfolioDays(
  days: LearningPortfolioDay[]
): LearningPortfolioDay[] {
  return days
    .map((day) => ({
      ...day,
      items: day.items.filter((item) => item.outcome !== "correct"),
    }))
    .filter((day) => day.items.length > 0)
}

export function buildLearningReviewItems(
  evidence: LearningEvidenceEntry[] | undefined,
  assessments: LearningDomainAssessment[] | undefined
): LearningReviewItem[] {
  const items = new Map<string, LearningReviewItem>()

  for (const entry of evidence ?? []) {
    if (!isReviewOutcome(entry.assessmentOutcome)) continue
    items.set(entry.id, {
      id: entry.id,
      conversationId: entry.conversationId,
      knowledgeCode: entry.knowledgeCode,
      outcome: entry.assessmentOutcome,
      questionText: entry.questionText,
      selectedChoiceText: entry.selectedChoiceText,
      createdAt: entry.createdAt,
    })
  }

  for (const assessment of assessments ?? []) {
    for (const entry of assessment.recentEvidence) {
      if (!isReviewOutcome(entry.outcome) || items.has(entry.evidenceId)) continue
      items.set(entry.evidenceId, {
        id: entry.evidenceId,
        conversationId: entry.conversationId,
        knowledgeCode: assessment.knowledgeCode,
        outcome: entry.outcome,
        questionText: entry.questionText,
        selectedChoiceText: entry.selectedChoiceText,
        createdAt: entry.createdAt,
      })
    }
  }

  return [...items.values()].sort((left, right) => {
    const outcomeOrder = outcomePriority(left.outcome) - outcomePriority(right.outcome)
    if (outcomeOrder !== 0) return outcomeOrder
    return Date.parse(right.createdAt) - Date.parse(left.createdAt)
  })
}

export function groupLearningPath(
  assessments: LearningDomainAssessment[] | undefined
): Record<LearningPathGroup, LearningDomainAssessment[]> {
  const groups: Record<LearningPathGroup, LearningDomainAssessment[]> = {
    priority: [],
    evidence: [],
    retention: [],
  }

  for (const assessment of assessments ?? []) {
    groups[getLearningPathGroup(assessment)].push(assessment)
  }

  for (const group of Object.values(groups)) {
    group.sort((left, right) => {
      const dateOrder =
        Date.parse(right.latestAssessedAt ?? "") -
        Date.parse(left.latestAssessedAt ?? "")
      return Number.isNaN(dateOrder) ? 0 : dateOrder
    })
  }

  return groups
}

export function getLearningPathGroup(
  assessment: LearningDomainAssessment
): LearningPathGroup {
  if (
    assessment.statusCode === "needs_attention" ||
    assessment.statusCode === "developing"
  ) {
    return "priority"
  }
  if (
    assessment.statusCode === "not_assessed" ||
    assessment.statusCode === "collecting_evidence"
  ) {
    return "evidence"
  }
  return "retention"
}

function isReviewOutcome(
  outcome: string | null | undefined
): outcome is "incorrect" | "partial" {
  return outcome === "incorrect" || outcome === "partial"
}

function isPortfolioOutcome(
  outcome: string | null | undefined
): outcome is LearningPortfolioItem["outcome"] {
  return (
    outcome === "correct" ||
    outcome === "partial" ||
    outcome === "incorrect"
  )
}

function outcomePriority(outcome: LearningReviewItem["outcome"]) {
  return outcome === "incorrect" ? 0 : 1
}
