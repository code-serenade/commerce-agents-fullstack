import { request } from "@workspace/services/api/base"
export interface LearningProfileResponse {
  frameworkVersion: string
  masteryMap: Record<string, unknown>
  engagementSummary: Record<string, unknown>
  coverageRate: number
  assessmentSummary?: LearningAssessmentSummary
  assessmentTimeline?: LearningAssessmentTimelinePoint[]
  domainAssessments?: LearningDomainAssessment[]
  analysisStatus: string
  lastAnalyzedAt?: string | null
  enabled: boolean
}

export interface LearningDomainAssessment {
  knowledgeCode: string
  statusCode:
    | "not_assessed"
    | "collecting_evidence"
    | "needs_attention"
    | "developing"
    | "demonstrated"
    | "retained"
  reasonCode: string
  observed: boolean
  observationCount: number
  assessmentCount: number
  distinctQuestionCount: number
  assessmentAngleCount: number
  correctCount: number
  partialCount: number
  incorrectCount: number
  latestOutcome?: LearningProbeOutcome | null
  latestAssessedAt?: string | null
  retentionVerified: boolean
  missingEvidenceCodes: string[]
  nextActionCode: string
  recentEvidence: LearningDomainEvidenceSummary[]
}

export interface LearningDomainEvidenceSummary {
  evidenceId: string
  conversationId: string
  questionId?: string | null
  questionText?: string | null
  selectedChoiceText?: string | null
  outcome: LearningProbeOutcome
  createdAt: string
}

export interface LearningAssessmentTimelinePoint {
  periodStart: string
  assessedKnowledgeCount: number
  demonstratedKnowledgeCount: number
  retainedKnowledgeCount: number
}

export interface LearningAssessmentSummary {
  assessedKnowledgeCount: number
  assessmentCount: number
  distinctAssessmentQuestionCount: number
  multiAngleKnowledgeCount: number
  demonstratedKnowledgeCount: number
  retainedKnowledgeCount?: number
  correctCount: number
  partialCount: number
  incorrectCount: number
  needsAttentionKnowledgeCount: number
}
export const getLearningProfileApi = (): Promise<LearningProfileResponse> =>
  request<undefined, LearningProfileResponse>({
    method: "GET",
    url: "/api/app/learning/profile",
  })

export interface LearningProbeResponse {
  id: string
  questionId?: string | null
  questionVersion?: number | null
  questionType: "single_choice" | "open_response"
  choices: LearningProbeChoice[]
  engaged: boolean
  knowledgeCode: string
  questionText: string
  reasonCode: string
  interactionMode: "contextual_micro"
  rewardCode: "progress_feedback"
  expiresAt: string
}

export interface LearningProbeChoice {
  id: string
  text: string
}

export type LearningProbeOutcome =
  | "correct"
  | "partial"
  | "incorrect"
  | "unscorable"
  | "legacy_unscored"

export interface LearningProbeSourceLabel {
  sourceId: string
  title: string
  publisher: string
  url: string
}

export interface LearningProbeResultResponse {
  id: string
  status: string
  canContinue?: boolean
  outcome?: LearningProbeOutcome | null
  score?: number | null
  feedbackCode?: string | null
  selectedChoiceId?: string | null
  selectedChoiceText?: string | null
  correctChoiceText?: string | null
  answerComparisonCode?:
    | "matches_reference"
    | "differs_from_reference"
    | "not_available"
    | null
  knowledgePoints?: string[]
  demonstratedPoints: string[]
  nextPoint?: string | null
  referenceExplanation?: string | null
  sourceLabels: LearningProbeSourceLabel[]
}

export interface LearningProbeHistoryEntry {
  probe: LearningProbeResponse
  result: LearningProbeResultResponse
}

export interface LearningProbeStatusResponse {
  id: string
  status: string
}

export interface LearningEvidenceEntry {
  id: string
  conversationId?: string
  probeId?: string | null
  questionText?: string | null
  selectedChoiceId?: string | null
  selectedChoiceText?: string | null
  knowledgeCode: string
  evidenceType: string
  direction: string
  strength: number
  summary: string
  assessmentOutcome?: string | null
  questionId?: string | null
  rubricVersion?: number | null
  createdAt: string
}

export type LearningEvidenceRange = "recent" | "30d" | "90d"

export interface LearningEvidenceQuery {
  limit?: number
  range?: LearningEvidenceRange
}

export interface NextLearningProbeQuery {
  conversationId?: string
  assistantMessageId?: string
  continueFromProbeId?: string
}

export interface UpdateLearningSettingsRequest {
  enabled: boolean
}

export const updateLearningSettingsApi = (
  body: UpdateLearningSettingsRequest
): Promise<LearningProfileResponse> =>
  request<UpdateLearningSettingsRequest, LearningProfileResponse>({
    method: "PATCH",
    url: "/api/app/learning/settings",
    body,
  })

export const getNextLearningProbeApi = (
  params: NextLearningProbeQuery = {}
): Promise<LearningProbeResponse | null> =>
  request<NextLearningProbeQuery, LearningProbeResponse | null>({
    method: "GET",
    url: "/api/app/learning/probes/next",
    body: params,
    suppressGlobalError: true,
  })

export const getLearningProbeHistoryApi = (params: {
  conversationId: string
  assistantMessageId: string
}): Promise<LearningProbeHistoryEntry[]> =>
  request<typeof params, LearningProbeHistoryEntry[]>({
    method: "GET",
    url: "/api/app/learning/probes/history",
    body: params,
    suppressGlobalError: true,
  })

export const skipLearningProbeApi = (
  probeId: string
): Promise<LearningProbeStatusResponse> =>
  request<undefined, LearningProbeStatusResponse>({
    method: "POST",
    url: `/api/app/learning/probes/${encodeURIComponent(probeId)}/skip`,
    suppressGlobalError: true,
  })

export const engageLearningProbeApi = (
  probeId: string
): Promise<LearningProbeStatusResponse> =>
  request<undefined, LearningProbeStatusResponse>({
    method: "POST",
    url: `/api/app/learning/probes/${encodeURIComponent(probeId)}/engage`,
    suppressGlobalError: true,
  })

export interface SubmitLearningProbeChoiceRequest {
  choiceId: string
}

export const submitLearningProbeChoiceApi = (
  probeId: string,
  body: SubmitLearningProbeChoiceRequest
): Promise<LearningProbeResultResponse> =>
  request<SubmitLearningProbeChoiceRequest, LearningProbeResultResponse>({
    method: "POST",
    url: `/api/app/learning/probes/${encodeURIComponent(probeId)}/answer-choice`,
    body,
    suppressGlobalError: true,
  })

export const getLearningProbeResultApi = (
  probeId: string
): Promise<LearningProbeResultResponse> =>
  request<undefined, LearningProbeResultResponse>({
    method: "GET",
    url: `/api/app/learning/probes/${encodeURIComponent(probeId)}/result`,
    suppressGlobalError: true,
  })

export const getLearningEvidenceApi = (
  params: LearningEvidenceQuery = {}
): Promise<LearningEvidenceEntry[]> =>
  request<LearningEvidenceQuery, LearningEvidenceEntry[]>({
    method: "GET",
    url: "/api/app/learning/evidence",
    body: params,
  })
