import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { LearningProgressSummary } from "./learning-progress-summary"

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: { count?: number; total?: number }) =>
      values?.count !== undefined
        ? `${key} ${values.count} / ${values.total}`
        : key,
  }),
}))

describe("LearningProgressSummary", () => {
  it("shows spaced verification inside the existing assessment card", () => {
    const { container } = render(
      <LearningProgressSummary
        assessedCount={3}
        coverageRate={63}
        demonstratedCount={2}
        lastAnalyzedAt={null}
        multiAngleCount={2}
        observedCount={5}
        retainedCount={1}
        totalDomains={8}
      />
    )

    expect(
      screen.getByText("learningProgress.retained 1 / 8")
    ).toBeInTheDocument()
    expect(container.querySelectorAll("section")).toHaveLength(4)
  })
})
