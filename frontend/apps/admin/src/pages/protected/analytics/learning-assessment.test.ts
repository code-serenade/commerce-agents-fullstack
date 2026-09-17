import { describe, expect, it } from "vitest"
import {
  getLearningEvaluationStatus,
  normalizeLearningAssessmentSummary,
} from "./learning-assessment"

describe("learning assessment presentation", () => {
  it("falls back to zero for rolling deployments with missing fields", () => {
    expect(
      normalizeLearningAssessmentSummary({
        assessedKnowledgeCount: 2,
        assessmentCount: 5,
      })
    ).toEqual({
      assessedKnowledgeCount: 2,
      assessmentCount: 5,
      distinctAssessmentQuestionCount: 0,
      multiAngleKnowledgeCount: 0,
      demonstratedKnowledgeCount: 0,
      retainedKnowledgeCount: 0,
      correctCount: 0,
      partialCount: 0,
      incorrectCount: 0,
      needsAttentionKnowledgeCount: 0,
    })
  })

  it("does not convert an empty assessment into a mastery state", () => {
    const summary = normalizeLearningAssessmentSummary(undefined)
    expect(getLearningEvaluationStatus(summary)).toBe("unassessed")
  })

  it("prioritizes a current reinforcement need over positive aggregate signals", () => {
    const summary = normalizeLearningAssessmentSummary({
      assessedKnowledgeCount: 3,
      demonstratedKnowledgeCount: 2,
      retainedKnowledgeCount: 1,
      needsAttentionKnowledgeCount: 1,
    })
    expect(getLearningEvaluationStatus(summary)).toBe("needs_attention")
  })
})
