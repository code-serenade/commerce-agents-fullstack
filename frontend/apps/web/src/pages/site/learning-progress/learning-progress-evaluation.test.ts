import { describe, expect, it } from "vitest"
import {
  getAssessedKnowledgeCount,
  getDomainEvaluationStatus,
} from "./learning-progress-evaluation"
import type { LearningEvidenceEntry } from "@/api"

const evidence = (assessmentOutcome: string): LearningEvidenceEntry[] => [
  {
    id: "evidence-1",
    knowledgeCode: "conversion",
    evidenceType: "probe_answer",
    direction: "positive",
    strength: 1,
    summary: "test",
    assessmentOutcome,
    createdAt: "2026-08-25T00:00:00Z",
  },
]

describe("getDomainEvaluationStatus", () => {
  it("prefers the latest deterministic evidence over a stale profile status", () => {
    expect(
      getDomainEvaluationStatus(
        { evaluationStatus: "needs_attention" },
        evidence("correct"),
        "conversion"
      )
    ).toBe("assessed")
  })

  it.each([
    ["correct", "assessed"],
    ["partial", "developing"],
    ["incorrect", "needs_attention"],
  ])("maps recent %s evidence to %s", (outcome, status) => {
    expect(
      getDomainEvaluationStatus(undefined, evidence(outcome), "conversion")
    ).toBe(status)
  })

  it("ignores newer unscored observations when locating the latest assessment", () => {
    expect(
      getDomainEvaluationStatus(
        undefined,
        [
          ...evidence("observed_unscored"),
          {
            ...evidence("partial")[0],
            id: "evidence-2",
            createdAt: "2026-08-24T00:00:00Z",
          },
        ],
        "conversion"
      )
    ).toBe("developing")
  })

  it("falls back to the profile when no deterministic evidence is available", () => {
    expect(
      getDomainEvaluationStatus(
        { evaluationStatus: "needs_attention" },
        evidence("unscorable"),
        "conversion"
      )
    ).toBe("needs_attention")
  })
})

describe("getAssessedKnowledgeCount", () => {
  it("does not count natural chat observations as assessed domains", () => {
    expect(
      getAssessedKnowledgeCount({
        market: { evidenceCount: 3, status: "observed" },
        conversion: { assessedCount: 2, evaluationStatus: "assessed" },
        finance: { assessedCount: 1, evaluationStatus: "needs_attention" },
        invalid: "ignored",
      })
    ).toBe(2)
  })
})
