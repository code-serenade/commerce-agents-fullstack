import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import {
  LearningAssessmentTimeline,
} from "./learning-assessment-timeline"
import { buildLearningAssessmentTimelineRows } from "./learning-assessment-timeline-view-model"

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))

vi.mock("@workspace/ui-components/stable/chart", () => ({
  TrendLineChart: ({
    data,
    series,
  }: {
    data: Array<Record<string, number | string>>
    series: Array<{ key: string; label: string }>
  }) => (
    <div data-testid="stable-trend-chart">
      {series.map((item) => (
        <span key={item.key}>{item.label}</span>
      ))}
      <span>{String(data.at(-1)?.demonstrated)}</span>
    </div>
  ),
}))

const timeline = [
  {
    periodStart: "2026-08-17",
    assessedKnowledgeCount: 1,
    demonstratedKnowledgeCount: 1,
    retainedKnowledgeCount: 1,
  },
  {
    periodStart: "2026-08-24",
    assessedKnowledgeCount: 1,
    demonstratedKnowledgeCount: 0,
    retainedKnowledgeCount: 0,
  },
]

describe("LearningAssessmentTimeline", () => {
  it("keeps the section absent for an older service response", () => {
    render(<LearningAssessmentTimeline timeline={undefined} />)

    expect(
      screen.queryByText("learningProgress.assessmentTimeline.title")
    ).not.toBeInTheDocument()
  })

  it("shows an honest empty state when all weekly snapshots are zero", () => {
    render(
      <LearningAssessmentTimeline
        timeline={timeline.map((point) => ({
          ...point,
          assessedKnowledgeCount: 0,
          demonstratedKnowledgeCount: 0,
          retainedKnowledgeCount: 0,
        }))}
      />
    )

    expect(
      screen.getByText("learningProgress.assessmentTimeline.empty")
    ).toBeInTheDocument()
    expect(screen.queryByTestId("stable-trend-chart")).not.toBeInTheDocument()
  })

  it("uses the stable chart and preserves a later mastery decrease", () => {
    render(<LearningAssessmentTimeline timeline={timeline} />)

    expect(screen.getByTestId("stable-trend-chart")).toBeInTheDocument()
    expect(
      screen.getByText("learningProgress.assessmentTimeline.assessed")
    ).toBeInTheDocument()
    expect(
      screen.getByText("learningProgress.assessmentTimeline.demonstrated")
    ).toBeInTheDocument()
    expect(
      screen.getByText("learningProgress.assessmentTimeline.retained")
    ).toBeInTheDocument()
    expect(screen.getByText("0")).toBeInTheDocument()
  })

  it("maps the API points without changing their order or values", () => {
    const rows = buildLearningAssessmentTimelineRows(timeline)

    expect(rows).toHaveLength(2)
    expect(rows[0]).toMatchObject({ assessed: 1, demonstrated: 1, retained: 1 })
    expect(rows[1]).toMatchObject({ assessed: 1, demonstrated: 0, retained: 0 })
    expect(rows[0]?.period).not.toBe("")
  })
})
