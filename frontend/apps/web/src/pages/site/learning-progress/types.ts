import type {
  LearningDomainAssessment,
  LearningEvidenceEntry,
  LearningProfileResponse,
} from "@/api"

export type LearningProgressViewKey =
  | "overview"
  | "review"
  | "path"
  | "map"
  | "portfolio"

export interface LearningDomainView {
  code: string
  label: string
  observed: boolean
  evaluationStatus?: string
  assessment?: LearningDomainAssessment
}

export interface LearningReviewItem {
  id: string
  conversationId?: string
  knowledgeCode: string
  outcome: "incorrect" | "partial"
  questionText?: string | null
  selectedChoiceText?: string | null
  createdAt: string
}

export interface LearningCapabilityRow {
  knowledgeCode: string
  statusCode: LearningDomainAssessment["statusCode"]
  assessmentCount: number
  distinctQuestionCount: number
  assessmentAngleCount: number
  correctCount: number
  partialCount: number
  incorrectCount: number
  latestAssessedAt?: string | null
}

export interface LearningPortfolioItem {
  id: string
  conversationId?: string
  knowledgeCode: string
  outcome: "correct" | "partial" | "incorrect"
  questionText?: string | null
  selectedChoiceText?: string | null
  createdAt: string
}

export interface LearningPortfolioDay {
  date: string
  items: LearningPortfolioItem[]
}

export interface LearningProgressViewModel {
  assessedCount: number
  coverageRate: number
  demonstratedCount: number
  evidenceCount: number
  analyzedMessages: number
  multiAngleCount: number
  observedCount: number
  retainedCount: number
  domainAssessments: LearningDomainAssessment[]
  capabilityRows: LearningCapabilityRow[]
  portfolioDays: LearningPortfolioDay[]
  reviewDays: LearningPortfolioDay[]
  portfolioSummary: { total: number; correct: number; review: number }
  rawDomains: Array<{
    code: string
    observed: boolean
    evaluationStatus?: string
    assessment?: LearningDomainAssessment
  }>
}

export interface LearningProgressSource {
  evidence?: LearningEvidenceEntry[]
  profile?: LearningProfileResponse
}
