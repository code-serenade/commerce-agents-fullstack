import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { FilesTurnSelectionSummary } from "./files-turn-selection-summary"

const t = (key: string, options?: Record<string, unknown>) => {
  if (key === "commerce.files.turnSelection.title") {
    return `next:${String(options?.count)}`
  }
  if (key === "commerce.files.turnSelection.files") {
    return `files:${String(options?.names)} hidden:${String(options?.hidden)}`
  }
  if (key === "commerce.files.turnSelection.empty") return "empty"
  if (key === "commerce.files.turnSelection.action") return "show"
  if (key === "commerce.files.turnSelection.clearAction") return "clear"
  return key
}

describe("FilesTurnSelectionSummary", () => {
  it("shows turn actions and clears only the current turn source refs", async () => {
    const user = userEvent.setup()
    const onClearSelectedForTurn = vi.fn()
    const onShowSelectedForTurn = vi.fn()

    render(
      <FilesTurnSelectionSummary
        canClearSelectedForTurn
        onClearSelectedForTurn={onClearSelectedForTurn}
        onShowSelectedForTurn={onShowSelectedForTurn}
        summary={{
          hiddenNames: 1,
          selectedNames: ["brief.pdf", "sheet.xlsx"],
          total: 3,
        }}
        t={t}
      />
    )

    await user.click(screen.getByRole("button", { name: /show/i }))
    await user.click(screen.getByRole("button", { name: /clear/i }))

    expect(onShowSelectedForTurn).toHaveBeenCalledTimes(1)
    expect(onClearSelectedForTurn).toHaveBeenCalledTimes(1)
  })

  it("does not expose turn actions when no material is selected", () => {
    render(
      <FilesTurnSelectionSummary
        canClearSelectedForTurn={false}
        onClearSelectedForTurn={vi.fn()}
        onShowSelectedForTurn={vi.fn()}
        summary={{
          hiddenNames: 0,
          selectedNames: [],
          total: 0,
        }}
        t={t}
      />
    )

    expect(screen.queryByRole("button", { name: /show/i })).not.toBeInTheDocument()
    expect(screen.queryByRole("button", { name: /clear/i })).not.toBeInTheDocument()
    expect(screen.getByText("empty")).toBeInTheDocument()
  })
})
