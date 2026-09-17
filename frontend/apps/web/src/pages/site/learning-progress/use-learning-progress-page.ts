import { useState } from "react"
import { useSearchParams } from "react-router"
import type { LearningEvidenceRange } from "@/api"
import {
  useLearningEvidence,
  useLearningProfile,
  useUpdateLearningSettings,
} from "./learning-progress-data"
import type { LearningProgressViewKey } from "./types"

export function useLearningProgressPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const activeView = parseLearningProgressView(searchParams.get("view"))
  const [evidenceRange, setEvidenceRange] =
    useState<LearningEvidenceRange>("recent")
  const profileQuery = useLearningProfile()
  const evidenceQuery = useLearningEvidence(evidenceRange)
  const updateSettings = useUpdateLearningSettings()

  return {
    activeView,
    evidence: evidenceQuery.data,
    evidenceError: evidenceQuery.isError,
    evidenceLoading: evidenceQuery.isLoading,
    evidenceRange,
    isLoading: profileQuery.isLoading,
    loadError: profileQuery.isError,
    onEvidenceRangeChange: setEvidenceRange,
    onToggleTracking: (enabled: boolean) => updateSettings.mutate(enabled),
    onViewChange: (view: LearningProgressViewKey) => {
      setSearchParams(view === "overview" ? {} : { view }, { replace: true })
    },
    profile: profileQuery.data,
    trackingPending: updateSettings.isPending,
  }
}

export function parseLearningProgressView(
  value: string | null
): LearningProgressViewKey {
  return value === "review" ||
    value === "path" ||
    value === "map" ||
    value === "portfolio"
    ? value
    : "overview"
}
