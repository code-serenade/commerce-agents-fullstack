import type { LearningAssessmentSummary } from "../learning-assessment"

export interface LearningStudentRow {
  user_id: string
  display_id: string
  display_name: string
  school_name: string | null
  team_name: string | null
  coverage_rate: number
  observed_count: number
  mastery_status: string
  assessment_summary: LearningAssessmentSummary
  enabled: boolean
  last_analyzed_at: string | null
  createdAt: string | null
  updatedAt: string | null
}

export interface LearningStudentTableQuery {
  keyword: string
}

export interface LearningQuestionMetricRow {
  question_id: string
  question_version: number
  knowledge_code: string
  question_text: string
  jurisdiction_code: LearningQuestionJurisdictionCode
  platform_code: LearningQuestionPlatformCode
  business_stage_code: LearningQuestionBusinessStageCode
  source_publishers: string[]
  source_valid_through: string | null
  curriculum_review_required: boolean | null
  production_approved: boolean | null
  release_status: LearningQuestionReleaseStatus
  issued_count: number
  active_count: number
  answered_count: number
  skipped_count: number
  expired_count: number
  superseded_count: number
  correct_count: number
  partial_count: number
  incorrect_count: number
  unscorable_count: number
  answer_rate: number
  skip_rate: number
  correct_rate: number
  average_answer_seconds: number
  choice_metrics: LearningQuestionChoiceMetricRow[]
  unmapped_choice_count: number
}

export type LearningQuestionJurisdictionCode =
  | "not_region_specific"
  | "cn"
  | "eu"
  | "us"
  | "unknown"

export type LearningQuestionPlatformCode =
  | "platform_agnostic"
  | "google_trends"
  | "google_merchant_center"
  | "google_analytics_4"
  | "google_search"
  | "google_ads"
  | "unknown"

export type LearningQuestionBusinessStageCode =
  | "market_research"
  | "product_operations"
  | "content_operations"
  | "traffic_acquisition"
  | "conversion_optimization"
  | "financial_management"
  | "customer_service"
  | "compliance_governance"
  | "unknown"

export interface LearningQuestionChoiceMetricRow {
  choice_id: string
  choice_text: string
  selected_count: number
  selected_rate: number
}

export type LearningQuestionReleaseStatus =
  | "production_approved"
  | "curriculum_pending"
  | "internal_pilot"
  | "unknown"

export interface LearningQuestionTableQuery {
  keyword: string
  knowledgeCode: string
  releaseStatus: "" | LearningQuestionReleaseStatus
}
