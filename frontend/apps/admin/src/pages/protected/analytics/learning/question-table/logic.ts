import type { DataTableFetchResult, DataTableSortState } from "@workspace/app-kit"
import type { LearningProbeQuestionMetricsResponse } from "@/api"
import type {
  LearningQuestionBusinessStageCode,
  LearningQuestionJurisdictionCode,
  LearningQuestionMetricRow,
  LearningQuestionPlatformCode,
  LearningQuestionReleaseStatus,
  LearningQuestionTableQuery,
} from "../types"

const jurisdictionCodes = new Set<LearningQuestionJurisdictionCode>([
  "not_region_specific",
  "cn",
  "eu",
  "us",
])
const platformCodes = new Set<LearningQuestionPlatformCode>([
  "platform_agnostic",
  "google_trends",
  "google_merchant_center",
  "google_analytics_4",
  "google_search",
  "google_ads",
])
const businessStageCodes = new Set<LearningQuestionBusinessStageCode>([
  "market_research",
  "product_operations",
  "content_operations",
  "traffic_acquisition",
  "conversion_optimization",
  "financial_management",
  "customer_service",
  "compliance_governance",
])

function finiteOrZero(value: number | undefined) {
  return typeof value === "number" && Number.isFinite(value) ? value : 0
}

function optionalBoolean(value: boolean | undefined) {
  return typeof value === "boolean" ? value : null
}

function optionalText(value: string | null | undefined) {
  const normalized = value?.trim()
  return normalized ? normalized : null
}

function jurisdictionCode(value: string | undefined): LearningQuestionJurisdictionCode {
  const normalized = value?.trim() as LearningQuestionJurisdictionCode | undefined
  return normalized && jurisdictionCodes.has(normalized) ? normalized : "unknown"
}

function platformCode(value: string | undefined): LearningQuestionPlatformCode {
  const normalized = value?.trim() as LearningQuestionPlatformCode | undefined
  return normalized && platformCodes.has(normalized) ? normalized : "unknown"
}

function businessStageCode(
  value: string | undefined
): LearningQuestionBusinessStageCode {
  const normalized = value?.trim() as
    | LearningQuestionBusinessStageCode
    | undefined
  return normalized && businessStageCodes.has(normalized)
    ? normalized
    : "unknown"
}

function nonNegativeFiniteOrZero(value: number | undefined) {
  return Math.max(0, finiteOrZero(value))
}

function boundedRate(value: number | undefined) {
  return Math.min(1, nonNegativeFiniteOrZero(value))
}

export function getLearningQuestionReleaseStatus({
  curriculumReviewRequired,
  productionApproved,
}: {
  curriculumReviewRequired: boolean | null
  productionApproved: boolean | null
}): LearningQuestionReleaseStatus {
  if (productionApproved === true) return "production_approved"
  if (curriculumReviewRequired === true) return "curriculum_pending"
  if (productionApproved === null || curriculumReviewRequired === null) {
    return "unknown"
  }
  return "internal_pilot"
}

export function mapLearningQuestionMetricRow(
  question: LearningProbeQuestionMetricsResponse
): LearningQuestionMetricRow {
  const curriculumReviewRequired = optionalBoolean(
    question.curriculumReviewRequired
  )
  const productionApproved = optionalBoolean(question.productionApproved)
  const sourcePublishers = (question.sourcePublishers ?? [])
    .map((publisher) => publisher.trim())
    .filter(Boolean)
    .filter((publisher, index, publishers) =>
      publishers.indexOf(publisher) === index
    )
    .sort((left, right) => left.localeCompare(right))
  const seenChoiceIds = new Set<string>()
  const choiceMetrics = (question.choiceMetrics ?? []).flatMap((choice) => {
    const choiceId = choice.choiceId?.trim()
    if (!choiceId || seenChoiceIds.has(choiceId)) return []
    seenChoiceIds.add(choiceId)
    return [
      {
        choice_id: choiceId,
        choice_text: choice.choiceText?.trim() || choiceId,
        selected_count: nonNegativeFiniteOrZero(choice.selectedCount),
        selected_rate: boundedRate(choice.selectedRate),
      },
    ]
  })
  return {
    question_id: question.questionId.trim(),
    question_version: question.questionVersion,
    knowledge_code: question.knowledgeCode.trim(),
    question_text: question.questionText.trim(),
    jurisdiction_code: jurisdictionCode(question.jurisdictionCode),
    platform_code: platformCode(question.platformCode),
    business_stage_code: businessStageCode(question.businessStageCode),
    source_publishers: sourcePublishers,
    source_valid_through: optionalText(question.sourceValidThrough),
    curriculum_review_required: curriculumReviewRequired,
    production_approved: productionApproved,
    release_status: getLearningQuestionReleaseStatus({
      curriculumReviewRequired,
      productionApproved,
    }),
    issued_count: finiteOrZero(question.issuedCount),
    active_count: finiteOrZero(question.activeCount),
    answered_count: finiteOrZero(question.answeredCount),
    skipped_count: finiteOrZero(question.skippedCount),
    expired_count: finiteOrZero(question.expiredCount),
    superseded_count: finiteOrZero(question.supersededCount),
    correct_count: finiteOrZero(question.correctCount),
    partial_count: finiteOrZero(question.partialCount),
    incorrect_count: finiteOrZero(question.incorrectCount),
    unscorable_count: finiteOrZero(question.unscorableCount),
    answer_rate: finiteOrZero(question.answerRate),
    skip_rate: finiteOrZero(question.skipRate),
    correct_rate: finiteOrZero(question.correctRate),
    average_answer_seconds: finiteOrZero(question.averageAnswerSeconds),
    choice_metrics: choiceMetrics,
    unmapped_choice_count: nonNegativeFiniteOrZero(
      question.unmappedChoiceCount
    ),
  }
}

function compareRows(
  left: LearningQuestionMetricRow,
  right: LearningQuestionMetricRow,
  columnKey: string
) {
  switch (columnKey) {
    case "knowledgeCode":
      return left.knowledge_code.localeCompare(right.knowledge_code)
    case "questionId":
      return left.question_id.localeCompare(right.question_id)
    case "applicabilityScope":
      return applicabilitySortValue(left).localeCompare(
        applicabilitySortValue(right)
      )
    case "releaseStatus":
      return (
        releaseStatusRank(left.release_status) -
        releaseStatusRank(right.release_status)
      )
    case "sourcePublishers":
      return left.source_publishers
        .join(" ")
        .localeCompare(right.source_publishers.join(" "))
    case "sourceValidThrough":
      return (left.source_valid_through ?? "").localeCompare(
        right.source_valid_through ?? ""
      )
    case "issuedCount":
      return left.issued_count - right.issued_count
    case "answeredCount":
      return left.answered_count - right.answered_count
    case "skippedCount":
      return left.skipped_count - right.skipped_count
    case "answerRate":
      return left.answer_rate - right.answer_rate
    case "correctRate":
      return left.correct_rate - right.correct_rate
    case "averageAnswerSeconds":
      return left.average_answer_seconds - right.average_answer_seconds
    default:
      return 0
  }
}

function applicabilitySortValue(row: LearningQuestionMetricRow) {
  return [
    row.jurisdiction_code,
    row.platform_code,
    row.business_stage_code,
  ].join("|")
}

function releaseStatusRank(status: LearningQuestionReleaseStatus) {
  switch (status) {
    case "production_approved":
      return 3
    case "curriculum_pending":
      return 2
    case "internal_pilot":
      return 1
    case "unknown":
      return 0
  }
}

export function queryLearningQuestionRows(
  rows: LearningQuestionMetricRow[],
  page: number,
  pageSize: number,
  query: LearningQuestionTableQuery,
  sort: DataTableSortState | null
): DataTableFetchResult<LearningQuestionMetricRow> {
  const keyword = query.keyword.trim().toLocaleLowerCase()
  let filtered = rows.filter((row) => {
    const matchesDomain =
      !query.knowledgeCode || row.knowledge_code === query.knowledgeCode
    const matchesReleaseStatus =
      !query.releaseStatus || row.release_status === query.releaseStatus
    const matchesKeyword =
      !keyword ||
      row.question_id.toLocaleLowerCase().includes(keyword) ||
      row.question_text.toLocaleLowerCase().includes(keyword) ||
      row.knowledge_code.toLocaleLowerCase().includes(keyword) ||
      row.jurisdiction_code.toLocaleLowerCase().includes(keyword) ||
      row.platform_code.toLocaleLowerCase().includes(keyword) ||
      row.business_stage_code.toLocaleLowerCase().includes(keyword) ||
      row.source_publishers.some((publisher) =>
        publisher.toLocaleLowerCase().includes(keyword)
      )
    return matchesDomain && matchesReleaseStatus && matchesKeyword
  })

  if (sort) {
    const direction = sort.direction === "asc" ? 1 : -1
    filtered = [...filtered].sort(
      (left, right) => compareRows(left, right, sort.columnKey) * direction
    )
  }

  const safePageSize = Math.max(1, pageSize)
  const start = Math.max(0, page - 1) * safePageSize
  return {
    items: filtered.slice(start, start + safePageSize),
    total: filtered.length,
  }
}
