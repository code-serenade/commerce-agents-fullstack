import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import type { useLearningQuestionTable } from "../index"
import { LearningQuestionTableView } from "./question-table-view"

const { dataTableSpy } = vi.hoisted(() => ({ dataTableSpy: vi.fn() }))

vi.mock("@workspace/app-kit", () => ({
  DataTable: (props: unknown) => {
    dataTableSpy(props)
    return <div data-testid="question-table" />
  },
  MetricCards: () => <div data-testid="metric-cards" />,
}))

describe("LearningQuestionTableView", () => {
  it("keeps only the compact domain column sticky so mobile users can inspect later columns", () => {
    render(
      <LearningQuestionTableView
        metricCards={[]}
        table={{} as ReturnType<typeof useLearningQuestionTable>}
      />
    )

    expect(screen.getByTestId("question-table")).toBeInTheDocument()
    expect(dataTableSpy).toHaveBeenCalledWith(
      expect.objectContaining({ stickyColumns: { left: 1 } })
    )
  })
})
