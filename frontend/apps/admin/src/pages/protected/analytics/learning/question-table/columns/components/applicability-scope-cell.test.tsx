import { render, screen } from "@testing-library/react"
import type { TFunction } from "i18next"
import { describe, expect, it } from "vitest"
import type { LearningQuestionMetricRow } from "../../../types"
import { LearningQuestionApplicabilityScopeCell } from "./applicability-scope-cell"

const translations: Record<string, string> = {
  "admin.analytics.learning.questionTable.scopeLabels.jurisdiction": "Region",
  "admin.analytics.learning.questionTable.scopeLabels.platform": "Platform",
  "admin.analytics.learning.questionTable.scopeLabels.businessStage": "Stage",
  "admin.analytics.learning.questionTable.jurisdictions.cn": "China",
  "admin.analytics.learning.questionTable.platforms.google_analytics_4":
    "Google Analytics 4",
  "admin.analytics.learning.questionTable.businessStages.conversion_optimization":
    "Conversion optimization",
}
const t = ((key: string) => translations[key] ?? key) as TFunction

const row = {
  jurisdiction_code: "cn",
  platform_code: "google_analytics_4",
  business_stage_code: "conversion_optimization",
} as LearningQuestionMetricRow

describe("LearningQuestionApplicabilityScopeCell", () => {
  it("renders localized scope labels and values without exposing raw codes", () => {
    render(<LearningQuestionApplicabilityScopeCell row={row} t={t} />)

    expect(screen.getByText("Region")).toBeInTheDocument()
    expect(screen.getByText("Platform")).toBeInTheDocument()
    expect(screen.getByText("Stage")).toBeInTheDocument()
    expect(screen.getByText("China")).toBeInTheDocument()
    expect(screen.getByText("Google Analytics 4")).toBeInTheDocument()
    expect(screen.getByText("Conversion optimization")).toBeInTheDocument()
    expect(screen.queryByText("conversion_optimization")).not.toBeInTheDocument()
  })
})
