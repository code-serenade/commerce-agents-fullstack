import { describe, expect, it } from "vitest"
import type { LearningDomainAssessment } from "@/api"
import { getLearningEvaluationConditions } from "./learning-evaluation-rules"

function assessment(
  overrides: Partial<LearningDomainAssessment>
): LearningDomainAssessment {
  return {
    knowledgeCode: "conversion",
    statusCode: "not_assessed",
    reasonCode: "no_structured_assessment",
    observed: false,
    observationCount: 0,
    assessmentCount: 0,
    distinctQuestionCount: 0,
    assessmentAngleCount: 0,
    correctCount: 0,
    partialCount: 0,
    incorrectCount: 0,
    retentionVerified: false,
    missingEvidenceCodes: ["first_assessment"],
    nextActionCode: "answer_first_context_question",
    recentEvidence: [],
    ...overrides,
  }
}

describe("getLearningEvaluationConditions", () => {
  it.each([
    ["not_assessed", 0, 0, false, [false, false, false, false]],
    ["collecting_evidence", 1, 1, false, [true, false, false, false]],
    ["needs_attention", 3, 2, false, [true, true, false, false]],
    ["developing", 3, 2, false, [true, true, false, false]],
    ["demonstrated", 3, 2, false, [true, true, true, false]],
    ["retained", 4, 2, true, [true, true, true, true]],
  ] as const)(
    "explains the %s state with the same gates as the backend",
    (statusCode, assessmentCount, assessmentAngleCount, retained, expected) => {
      expect(
        getLearningEvaluationConditions(
          assessment({
            statusCode,
            assessmentCount,
            assessmentAngleCount,
            retentionVerified: retained,
          })
        ).map((condition) => condition.satisfied)
      ).toEqual(expected)
    }
  )
})
