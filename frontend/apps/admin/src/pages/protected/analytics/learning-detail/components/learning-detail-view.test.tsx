import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import type { LearningStudentDetailResponse } from "@/api"
import { LearningStudentDetailView } from "./learning-detail-view"

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

const detail: LearningStudentDetailResponse = {
  userId: "student-1",
  displayId: "S-001",
  displayName: "Student One",
  coverageRate: 0.25,
  observedKnowledgeCount: 2,
  masteryStatus: "observed",
  assessmentSummary: {
    assessedKnowledgeCount: 1,
    assessmentCount: 1,
    needsAttentionKnowledgeCount: 1,
  },
  enabled: true,
  masterySummary: [
    { knowledgeCode: "market", status: "observed", strength: 0.5 },
  ],
  evidenceSummary: [
    {
      id: "structured",
      knowledgeCode: "market",
      evidenceType: "probe_answer",
      direction: "negative",
      strength: 0,
      summary: "Structured summary should not replace explicit fields",
      assessmentOutcome: "incorrect",
      questionText: "Which signal does Google Trends provide?",
      selectedChoiceText: "Confirmed sales volume",
    },
    {
      id: "observation",
      knowledgeCode: "market",
      evidenceType: "explanation",
      direction: "neutral",
      strength: 0.4,
      summary: "Student discussed market interest.",
      assessmentOutcome: "observed_unscored",
    },
  ],
}

describe("LearningStudentDetailView", () => {
  it("shows structured questions and choices separately from observations", () => {
    render(
      <LearningStudentDetailView
        detail={detail}
        isLoading={false}
        onBack={vi.fn()}
      />
    )

    expect(
      screen.getByText("admin.analytics.learning.detail.fields.question")
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        "admin.analytics.learning.detail.fields.selectedChoice"
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText("Which signal does Google Trends provide?")
    ).toBeInTheDocument()
    expect(screen.getByText("Confirmed sales volume")).toBeInTheDocument()
    expect(
      screen.getByText("Student discussed market interest.")
    ).toBeInTheDocument()
    expect(
      screen.queryByText(
        "Structured summary should not replace explicit fields"
      )
    ).not.toBeInTheDocument()
  })
})
