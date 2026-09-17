import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { LearningChoiceDistributionCell } from "./choice-distribution-cell"

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: Record<string, number>) => {
      if (key.endsWith("choiceSelection")) {
        return `${values?.count ?? 0} · ${values?.rate ?? 0}%`
      }
      if (key.endsWith("unmappedChoices")) {
        return `${values?.count ?? 0} historical responses could not be mapped`
      }
      if (key.endsWith("noChoiceResponses")) return "No valid option responses"
      return key
    },
  }),
}))

describe("learning option distribution cell", () => {
  it("shows option order and neutral aggregate values without answer hints", () => {
    render(
      <LearningChoiceDistributionCell
        choices={[
          {
            choice_id: "view_item",
            choice_text: "view_item",
            selected_count: 1,
            selected_rate: 0.25,
          },
          {
            choice_id: "begin_checkout",
            choice_text: "begin_checkout",
            selected_count: 3,
            selected_rate: 0.75,
          },
        ]}
        unmappedChoiceCount={1}
      />
    )

    expect(screen.getByText("A")).toBeInTheDocument()
    expect(screen.getByText("B")).toBeInTheDocument()
    expect(screen.getByText("view_item")).toBeInTheDocument()
    expect(screen.getByText("begin_checkout")).toBeInTheDocument()
    expect(screen.getAllByRole("progressbar")).toHaveLength(2)
    expect(
      screen.getByText("1 historical responses could not be mapped")
    ).toBeInTheDocument()
    expect(screen.queryByText(/correct answer/i)).not.toBeInTheDocument()
  })

  it("keeps an older response without option metrics readable", () => {
    render(
      <LearningChoiceDistributionCell choices={[]} unmappedChoiceCount={0} />
    )

    expect(screen.getByText("No valid option responses")).toBeInTheDocument()
  })
})
