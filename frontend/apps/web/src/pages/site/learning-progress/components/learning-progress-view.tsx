import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import { PageErrorState, PageLoading } from "@workspace/app-kit"
import type {
  LearningEvidenceEntry,
  LearningEvidenceRange,
  LearningProfileResponse,
} from "@/api"
import type { LearningProgressViewKey } from "../types"
import { buildLearningProgressViewModel } from "../view-model/learning-progress-view-model"
import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../../commerce/components/commerce-ui"
import { LearningOverviewPanel } from "./learning-overview-panel"
import { LearningCapabilityMap } from "./learning-capability-map"
import { LearningPathPanel } from "./learning-path-panel"
import { LearningPortfolio } from "./learning-portfolio"
import { LearningProgressNavigation } from "./learning-progress-navigation"
export function LearningProgressView({
  activeView,
  evidence,
  evidenceError,
  evidenceLoading,
  evidenceRange,
  isLoading,
  loadError,
  onEvidenceRangeChange,
  onToggleTracking,
  onViewChange,
  profile,
  trackingPending,
}: {
  activeView: LearningProgressViewKey
  evidence?: LearningEvidenceEntry[]
  evidenceError: boolean
  evidenceLoading: boolean
  evidenceRange: LearningEvidenceRange
  isLoading: boolean
  loadError: boolean
  onEvidenceRangeChange: (range: LearningEvidenceRange) => void
  onToggleTracking: (enabled: boolean) => void
  onViewChange: (view: LearningProgressViewKey) => void
  profile?: LearningProfileResponse
  trackingPending: boolean
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const model = buildLearningProgressViewModel(profile, evidence ?? [])
  const domains = model.rawDomains.map((domain) => ({
    ...domain,
    label: t(`learningProgress.domains.${domain.code}`),
  }))
  const onContinueChat = () => navigate("/commerce")
  const onOpenConversation = (conversationId: string) =>
    navigate(`/commerce/${conversationId}`)
  if (isLoading) {
    return <PageLoading label={t("learningProgress.loading")} />
  }

  if (loadError) {
    return (
      <PageErrorState
        title={t("learningProgress.loadErrorTitle")}
        message={t("learningProgress.loadErrorDescription")}
      />
    )
  }

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("learningProgress.eyebrow")}
        title={t("learningProgress.title")}
        description={t("learningProgress.description")}
      />
      <CommerceBody className="px-4 py-5 sm:px-6">
        <div className="mx-auto w-full max-w-6xl space-y-4">
          <LearningProgressNavigation
            activeView={activeView}
            onViewChange={onViewChange}
          />
          {activeView === "overview" ? (
            <LearningOverviewPanel
              domains={domains}
              evidence={evidence}
              evidenceError={evidenceError}
              evidenceLoading={evidenceLoading}
              evidenceRange={evidenceRange}
              model={model}
              onContinueChat={onContinueChat}
              onEvidenceRangeChange={onEvidenceRangeChange}
              onOpenConversation={onOpenConversation}
              onToggleTracking={onToggleTracking}
              profile={profile}
              trackingPending={trackingPending}
            />
          ) : null}
          {activeView === "path" ? (
            <LearningPathPanel
              assessments={model.domainAssessments}
              onContinueChat={onContinueChat}
              onOpenConversation={onOpenConversation}
            />
          ) : null}
          {activeView === "map" ? (
            <LearningCapabilityMap
              rows={model.capabilityRows}
              onContinueChat={onContinueChat}
            />
          ) : null}
          {activeView === "portfolio" || activeView === "review" ? (
            <LearningPortfolio
              days={activeView === "review" ? model.reviewDays : model.portfolioDays}
              summary={model.portfolioSummary}
              evidenceRange={evidenceRange}
              onEvidenceRangeChange={onEvidenceRangeChange}
              reviewOnly={activeView === "review"}
              onReviewOnlyChange={(reviewOnly) =>
                onViewChange(reviewOnly ? "review" : "portfolio")
              }
              hasError={evidenceError}
              isLoading={evidenceLoading}
              onContinueChat={onContinueChat}
              onOpenConversation={onOpenConversation}
            />
          ) : null}
        </div>
      </CommerceBody>
    </CommerceSurface>
  )
}
