import { describe, expect, it } from "vitest"
import type { LearningProbeQuestionMetricsResponse } from "@/api"
import {
  mapLearningQuestionMetricRow,
  queryLearningQuestionRows,
} from "./logic"

function question(
  id: string,
  knowledgeCode: string,
  questionText: string,
  correctRate: number
): LearningProbeQuestionMetricsResponse {
  return {
    questionId: id,
    questionVersion: 1,
    knowledgeCode,
    questionText,
    jurisdictionCode: "not_region_specific",
    platformCode: "platform_agnostic",
    businessStageCode: "market_research",
    sourcePublishers: ["Official Publisher"],
    sourceValidThrough: "2027-02-27",
    curriculumReviewRequired: true,
    productionApproved: false,
    issuedCount: 4,
    answeredCount: 3,
    skippedCount: 1,
    answerRate: 0.75,
    correctRate,
    averageAnswerSeconds: 30,
    choiceMetrics: [
      {
        choiceId: "choice_a",
        choiceText: "Choice A",
        selectedCount: 2,
        selectedRate: 2 / 3,
      },
      {
        choiceId: "choice_b",
        choiceText: "Choice B",
        selectedCount: 1,
        selectedRate: 1 / 3,
      },
    ],
    unmappedChoiceCount: 0,
  }
}

describe("learning question quality table", () => {
  it("maps optional service counters to stable zeroes", () => {
    const row = mapLearningQuestionMetricRow({
      questionId: " market.q1 ",
      questionVersion: 1,
      knowledgeCode: " market ",
      questionText: " Question one ",
    })

    expect(row.question_id).toBe("market.q1")
    expect(row.knowledge_code).toBe("market")
    expect(row.question_text).toBe("Question one")
    expect(row.jurisdiction_code).toBe("unknown")
    expect(row.platform_code).toBe("unknown")
    expect(row.business_stage_code).toBe("unknown")
    expect(row.source_publishers).toEqual([])
    expect(row.source_valid_through).toBeNull()
    expect(row.release_status).toBe("unknown")
    expect(row.issued_count).toBe(0)
    expect(row.correct_rate).toBe(0)
    expect(row.choice_metrics).toEqual([])
    expect(row.unmapped_choice_count).toBe(0)
  })

  it("keeps option order and normalizes malformed distribution values", () => {
    const response = question(
      "market.q1",
      "market",
      "Research customers",
      0.5
    )
    response.choiceMetrics = [
      {
        choiceId: " choice_b ",
        choiceText: " Choice B ",
        selectedCount: 2,
        selectedRate: 1.5,
      },
      {
        choiceId: "choice_a",
        choiceText: "",
        selectedCount: -1,
        selectedRate: -0.2,
      },
      {
        choiceId: "choice_b",
        choiceText: "Duplicate",
        selectedCount: 99,
        selectedRate: 1,
      },
      {
        choiceId: " ",
        choiceText: "Missing ID",
        selectedCount: 1,
        selectedRate: 0.5,
      },
    ]
    response.unmappedChoiceCount = -4

    const row = mapLearningQuestionMetricRow(response)

    expect(row.choice_metrics).toEqual([
      {
        choice_id: "choice_b",
        choice_text: "Choice B",
        selected_count: 2,
        selected_rate: 1,
      },
      {
        choice_id: "choice_a",
        choice_text: "choice_a",
        selected_count: 0,
        selected_rate: 0,
      },
    ])
    expect(row.unmapped_choice_count).toBe(0)
  })

  it("searches, filters, sorts, and paginates the fixed question catalog locally", () => {
    const rows = [
      question("market.q2", "market", "Analyze competitors", 0.5),
      question("market.q1", "market", "Research customers", 0.9),
      question("product.q1", "product", "Manage products", 0.7),
    ].map(mapLearningQuestionMetricRow)

    const result = queryLearningQuestionRows(
      rows,
      1,
      1,
      { keyword: "q", knowledgeCode: "market", releaseStatus: "" },
      { columnKey: "correctRate", direction: "desc" }
    )

    expect(result.total).toBe(2)
    expect(result.items.map((row) => row.question_id)).toEqual(["market.q1"])
  })

  it("matches question text without changing backend rates", () => {
    const rows = [
      question("market.q1", "market", "Research customers", 0.6667),
      question("product.q1", "product", "Manage products", 0.2),
    ].map(mapLearningQuestionMetricRow)

    const result = queryLearningQuestionRows(
      rows,
      1,
      15,
      { keyword: "customer", knowledgeCode: "", releaseStatus: "" },
      null
    )

    expect(result.total).toBe(1)
    expect(result.items[0]?.correct_rate).toBe(0.6667)
  })

  it("searches publishers and filters the validated release status", () => {
    const pending = question(
      "market.q1",
      "market",
      "Research customers",
      0.8
    )
    pending.sourcePublishers = [" Australian Government ", "Australian Government"]
    const approved = question(
      "product.q1",
      "product",
      "Manage products",
      0.6
    )
    approved.productionApproved = true

    const rows = [pending, approved].map(mapLearningQuestionMetricRow)
    const result = queryLearningQuestionRows(
      rows,
      1,
      15,
      {
        keyword: "australian",
        knowledgeCode: "",
        releaseStatus: "curriculum_pending",
      },
      { columnKey: "sourceValidThrough", direction: "asc" }
    )

    expect(result.total).toBe(1)
    expect(result.items[0]?.source_publishers).toEqual([
      "Australian Government",
    ])
    expect(result.items[0]?.release_status).toBe("curriculum_pending")
  })

  it("normalizes, searches, and sorts machine-readable applicability scope", () => {
    const ga4 = question(
      "conversion.q1",
      "conversion",
      "Measure checkout",
      0.8
    )
    ga4.platformCode = "google_analytics_4"
    ga4.businessStageCode = "conversion_optimization"
    const china = question(
      "compliance.q1",
      "compliance",
      "Protect consumer choice",
      0.6
    )
    china.jurisdictionCode = "cn"
    china.businessStageCode = "compliance_governance"
    const malformed = question(
      "market.q1",
      "market",
      "Research demand",
      0.4
    )
    malformed.jurisdictionCode = "worldwide"
    malformed.platformCode = "unknown_vendor"
    malformed.businessStageCode = "sales"

    const rows = [ga4, china, malformed].map(mapLearningQuestionMetricRow)
    expect(rows[2]).toMatchObject({
      jurisdiction_code: "unknown",
      platform_code: "unknown",
      business_stage_code: "unknown",
    })

    const searched = queryLearningQuestionRows(
      rows,
      1,
      15,
      { keyword: "google_analytics_4", knowledgeCode: "", releaseStatus: "" },
      null
    )
    expect(searched.items.map((row) => row.question_id)).toEqual([
      "conversion.q1",
    ])

    const sorted = queryLearningQuestionRows(
      rows,
      1,
      15,
      { keyword: "", knowledgeCode: "", releaseStatus: "" },
      { columnKey: "applicabilityScope", direction: "asc" }
    )
    expect(sorted.items.map((row) => row.question_id)).toEqual([
      "compliance.q1",
      "conversion.q1",
      "market.q1",
    ])
  })
})
