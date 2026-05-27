import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"
import {
  DispatchFactChips,
  DispatchStepDock,
} from "./agent-dispatch-workbench-parts"

const factLabels = [
  "Plan",
  "Role audit",
  "Search audit",
  "Material audit",
  "Material note",
  "Not in context",
] as const
const longAuditDetail =
  "missing Ops extra Search planned Data, Ops executed Data; selected file-a file-b missing file-b; requested search_competitor"
const longAuditLabel = "Role audit"
const longStepDetail =
  "load selected file-a file-b, verify workspace pack readiness, execute search_competitor, then hand off evidence to the selected roles"
const stepTitle = "Material and search check"

describe("DispatchFactChips", () => {
  it("renders every selected dispatch fact from the view model", () => {
    render(
      <DispatchFactChips
        facts={factLabels.map((label, index) => ({
          detail: `detail ${index + 1}`,
          label,
          tone: index === 3 ? "warning" : "success",
        }))}
      />
    )

    factLabels.forEach((label) => {
      expect(screen.getByText(label)).toBeInTheDocument()
    })
  })

  it("wraps long audit details instead of truncating them", () => {
    render(
      <DispatchFactChips
        facts={[
          {
            detail: longAuditDetail,
            label: longAuditLabel,
            tone: "warning",
          },
        ]}
      />
    )

    const detail = screen.getByText(longAuditDetail)

    expect(detail).toHaveClass("wrap-break-word")
    expect(detail).not.toHaveClass("truncate")
  })
})

describe("DispatchStepDock", () => {
  it("shows step details in the persistent process dock", () => {
    render(
      <DispatchStepDock
        steps={[
          {
            detail: longStepDetail,
            id: "material-search-check",
            index: 0,
            state: "working",
            title: stepTitle,
          },
        ]}
      />
    )

    const detail = screen.getByText(longStepDetail)

    expect(screen.getByText(stepTitle)).toBeInTheDocument()
    expect(detail).toHaveClass("wrap-break-word")
    expect(detail).not.toHaveClass("truncate")
  })
})
