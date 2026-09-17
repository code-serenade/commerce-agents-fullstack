import type { TFunction } from "i18next"
import { describe, expect, it } from "vitest"
import {
  buildLearningMetricCards,
  buildLearningPilotMetricCards,
  buildLearningQuestionGovernanceMetricCards,
} from "./index"

const t = ((key: string) => key) as TFunction

describe("learning metric cards", () => {
  it("keeps legacy metrics and falls back new rolling fields to zero", () => {
    const cards = buildLearningMetricCards(
      {
        total: 12,
        enabled: 10,
        observed: 7,
        avgCoverageRate: 0.25,
      },
      t
    )

    expect(cards.map((card) => card.key)).toEqual([
      "total",
      "enabled",
      "observed",
      "assessed",
      "demonstrated",
      "needsAttention",
    ])
    expect(cards.find((card) => card.key === "assessed")?.value).toBe("0")
    expect(cards.find((card) => card.key === "demonstrated")?.value).toBe(
      "0"
    )
    expect(cards.find((card) => card.key === "needsAttention")?.value).toBe(
      "0"
    )
  })

  it("keeps pilot metrics compatible with an older metrics response", () => {
    const cards = buildLearningPilotMetricCards(
      {
        total: 12,
        enabled: 10,
        observed: 7,
        avgCoverageRate: 0.25,
      },
      t
    )

    expect(cards.map((card) => card.key)).toEqual([
      "issuedProbes",
      "answeredProbes",
      "skippedProbes",
      "correctProbes",
    ])
    expect(cards.map((card) => card.value)).toEqual(["0", "0", "0", "0"])
  })

  it("maps service pilot counts without recomputing their business rates", () => {
    const cards = buildLearningPilotMetricCards(
      {
        total: 12,
        enabled: 10,
        observed: 7,
        avgCoverageRate: 0.25,
        probePilot: {
          issuedCount: 18,
          answeredCount: 10,
          skippedCount: 3,
          correctCount: 7,
          answerRate: 0.6667,
          skipRate: 0.2,
          correctRate: 0.7,
          averageAnswerSeconds: 42,
        },
      },
      t
    )

    expect(cards.map((card) => card.value)).toEqual(["18", "10", "3", "7"])
  })

  it("keeps question governance unknown for an older metrics response", () => {
    const cards = buildLearningQuestionGovernanceMetricCards(
      {
        total: 12,
        enabled: 10,
        observed: 7,
        avgCoverageRate: 0.25,
      },
      t
    )

    expect(cards.map((card) => card.value)).toEqual([
      "-",
      "-",
      "-",
      "-",
      "-",
      "-",
      "-",
    ])
  })

  it("maps the service-owned question governance summary", () => {
    const cards = buildLearningQuestionGovernanceMetricCards(
      {
        total: 12,
        enabled: 10,
        observed: 7,
        avgCoverageRate: 0.25,
        probePilot: {
          questionBankVersion: "ecommerce_diagnostic_v10",
          questionCount: 82,
          assessmentAngleCount: 64,
          singleQuestionAngleCount: 48,
          productionApprovedCount: 0,
          curriculumReviewPendingCount: 82,
          earliestSourceValidThrough: "2027-02-27",
          coverageGapCount: 3,
          coverageGapTurnCount: 3,
          coverageGapDomainMetrics: [{ code: "product", count: 2 }],
          coverageGapReasonMetrics: [
            { code: "no_contextual_question", count: 3 },
          ],
        },
      },
      t
    )

    expect(cards.map((card) => card.value)).toEqual([
      "v10",
      "82",
      "64",
      "0",
      "82",
      "2027-02-27",
      "3",
    ])
    expect(cards.find((card) => card.key === "productionApprovedCount")?.variant).toBe(
      "warning"
    )
    expect(cards.find((card) => card.key === "assessmentAngleCount")?.variant).toBe(
      "warning"
    )
  })
})
