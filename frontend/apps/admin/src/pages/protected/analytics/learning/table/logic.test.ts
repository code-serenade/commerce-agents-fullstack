import { describe, expect, it } from "vitest"
import type { LearningStudentResponse } from "@/api"
import { mapLearningStudentRow } from "./logic"

const baseStudent: LearningStudentResponse = {
  userId: "student-1",
  displayId: "S-001",
  displayName: "Student One",
  coverageRate: 0.25,
  observedKnowledgeCount: 2,
  masteryStatus: "observed",
  enabled: true,
}

describe("learning student row mapping", () => {
  it("keeps conversation observation separate from missing assessment data", () => {
    const row = mapLearningStudentRow(baseStudent)

    expect(row.observed_count).toBe(2)
    expect(row.assessment_summary.assessedKnowledgeCount).toBe(0)
    expect(row.assessment_summary.assessmentCount).toBe(0)
  })

  it("maps the additive structured assessment summary", () => {
    const row = mapLearningStudentRow({
      ...baseStudent,
      assessmentSummary: {
        assessedKnowledgeCount: 1,
        assessmentCount: 3,
        demonstratedKnowledgeCount: 1,
      },
    })

    expect(row.assessment_summary.assessedKnowledgeCount).toBe(1)
    expect(row.assessment_summary.assessmentCount).toBe(3)
    expect(row.assessment_summary.demonstratedKnowledgeCount).toBe(1)
  })
})
