import type { LearningDomainAssessment } from "@/api"

export type LearningEvaluationConditionCode =
  | "structured_assessment"
  | "multiple_angles"
  | "positive_multi_angle"
  | "spaced_verification"

export interface LearningEvaluationCondition {
  code: LearningEvaluationConditionCode
  satisfied: boolean
  value?: number
  target?: number
}

export function getLearningEvaluationConditions(
  assessment: LearningDomainAssessment
): LearningEvaluationCondition[] {
  const positiveMultiAngle =
    assessment.statusCode === "demonstrated" ||
    assessment.statusCode === "retained"

  return [
    {
      code: "structured_assessment",
      satisfied: assessment.assessmentCount > 0,
      value: assessment.assessmentCount,
      target: 1,
    },
    {
      code: "multiple_angles",
      satisfied: assessment.assessmentAngleCount >= 2,
      value: assessment.assessmentAngleCount,
      target: 2,
    },
    {
      code: "positive_multi_angle",
      satisfied: positiveMultiAngle,
    },
    {
      code: "spaced_verification",
      satisfied: assessment.retentionVerified,
    },
  ]
}
