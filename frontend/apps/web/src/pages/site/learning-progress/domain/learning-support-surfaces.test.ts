import { describe, expect, it } from "vitest"
import type { LearningDomainAssessment, LearningEvidenceEntry } from "@/api"
import {
  buildLearningCapabilityRows,
  buildLearningPortfolioDays,
  filterLearningPortfolioDays,
  buildLearningReviewItems,
  groupLearningPath,
} from "./learning-support-surfaces"

describe("learning support surfaces", () => {
  it("does not mix profile evidence into an explicit range result", () => {
    const assessment = createAssessment("needs_attention", "product")
    assessment.recentEvidence = [{
      evidenceId: "outside-range",
      conversationId: "conversation-1",
      outcome: "incorrect",
      createdAt: "2026-01-01T08:00:00Z",
    }]
    expect(buildLearningPortfolioDays([], [assessment])).toEqual([])
    expect(buildLearningPortfolioDays(undefined, [assessment])).toHaveLength(1)
  })

  it("removes complete answers and empty dates from reinforcement view", () => {
    const entry = createEvidence("correct", "partial", "2026-09-17T08:00:00Z")
    entry.assessmentOutcome = "correct"
    const days = buildLearningPortfolioDays([
      entry,
      createEvidence("incorrect", "incorrect", "2026-09-16T08:00:00Z"),
    ], [])
    expect(filterLearningPortfolioDays(days).map((day) => day.date)).toEqual([
      "2026-09-16",
    ])
  })
  it("prioritizes incorrect records and removes duplicate domain evidence", () => {
    const evidence: LearningEvidenceEntry[] = [
      createEvidence("partial-1", "partial", "2026-09-15T08:00:00Z"),
      createEvidence("incorrect-1", "incorrect", "2026-09-14T08:00:00Z"),
    ]
    const assessment = createAssessment("needs_attention", "product")
    assessment.recentEvidence = [
      {
        evidenceId: "incorrect-1",
        conversationId: "conversation-duplicate",
        outcome: "incorrect",
        createdAt: "2026-09-14T08:00:00Z",
      },
      {
        evidenceId: "incorrect-2",
        conversationId: "conversation-2",
        outcome: "incorrect",
        createdAt: "2026-09-16T08:00:00Z",
      },
    ]

    const items = buildLearningReviewItems(evidence, [assessment])

    expect(items.map((item) => item.id)).toEqual([
      "incorrect-2",
      "incorrect-1",
      "partial-1",
    ])
  })

  it("groups domain assessments by the next learning purpose", () => {
    const groups = groupLearningPath([
      createAssessment("needs_attention", "product"),
      createAssessment("collecting_evidence", "content"),
      createAssessment("demonstrated", "conversion"),
    ])

    expect(groups.priority.map((item) => item.knowledgeCode)).toEqual([
      "product",
    ])
    expect(groups.evidence.map((item) => item.knowledgeCode)).toEqual([
      "content",
    ])
    expect(groups.retention.map((item) => item.knowledgeCode)).toEqual([
      "conversion",
    ])
  })

  it("keeps all curriculum domains in the capability map", () => {
    const rows = buildLearningCapabilityRows(
      ["market", "product"],
      [createAssessment("demonstrated", "product")]
    )

    expect(rows).toHaveLength(2)
    expect(rows[0]).toMatchObject({
      knowledgeCode: "market",
      statusCode: "not_assessed",
      assessmentCount: 0,
    })
    expect(rows[1]).toMatchObject({
      knowledgeCode: "product",
      statusCode: "demonstrated",
      assessmentCount: 1,
    })
  })

  it("groups only scorable structured answers into portfolio days", () => {
    const observed = createEvidence(
      "observed",
      "partial",
      "2026-09-16T09:00:00Z"
    )
    observed.assessmentOutcome = null
    const correct = createEvidence(
      "correct",
      "partial",
      "2026-09-16T08:00:00Z"
    )
    correct.assessmentOutcome = "correct"
    const incorrect = createEvidence(
      "incorrect",
      "incorrect",
      "2026-09-15T08:00:00Z"
    )

    const days = buildLearningPortfolioDays(
      [observed, incorrect, correct],
      []
    )

    expect(days.map((day) => day.date)).toEqual([
      "2026-09-16",
      "2026-09-15",
    ])
    expect(days.flatMap((day) => day.items).map((item) => item.id)).toEqual([
      "correct",
      "incorrect",
    ])
  })
})

function createEvidence(
  id: string,
  assessmentOutcome: "incorrect" | "partial",
  createdAt: string
): LearningEvidenceEntry {
  return {
    id,
    knowledgeCode: "product",
    evidenceType: "probe_answer",
    direction: "negative",
    strength: 1,
    summary: id,
    assessmentOutcome,
    createdAt,
  }
}

function createAssessment(
  statusCode: LearningDomainAssessment["statusCode"],
  knowledgeCode: string
): LearningDomainAssessment {
  return {
    knowledgeCode,
    statusCode,
    reasonCode: "latest_answer_incorrect",
    observed: true,
    observationCount: 1,
    assessmentCount: 1,
    distinctQuestionCount: 1,
    assessmentAngleCount: 1,
    correctCount: 0,
    partialCount: 0,
    incorrectCount: 1,
    latestOutcome: "incorrect",
    latestAssessedAt: "2026-09-16T08:00:00Z",
    retentionVerified: false,
    missingEvidenceCodes: ["positive_correction"],
    nextActionCode: "correct_latest_gap",
    recentEvidence: [],
  }
}
