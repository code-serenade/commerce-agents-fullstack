import type { LearningStudentSortBy } from "@/api"

const learningSortByValues: readonly LearningStudentSortBy[] = [
  "coverageRate",
  "observedKnowledgeCount",
  "lastAnalyzedAt",
  "updatedAt",
]

export function isLearningSortBy(
  value: string
): value is LearningStudentSortBy {
  return (learningSortByValues as readonly string[]).includes(value)
}
