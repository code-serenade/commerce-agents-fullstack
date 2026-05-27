import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"
import type { InlineAgent } from "../view-model/thread-inline-view-model"
import {
  InlineAgentGrid,
  InlineDispatchFactGrid,
} from "./inline-swarm-task-details"

const agentNames = [
  "Data agent",
  "Ops agent",
  "Search agent",
  "Creative agent",
  "Accounting agent",
] as const
const agentSummary = "Completed audit work"
const agentStatus = "Done"
const longAuditDetail =
  "missing Ops extra Search planned Data, Ops executed Data; selected file-a file-b missing file-b; requested search_competitor"
const longAuditLabel = "Role audit"

describe("InlineAgentGrid", () => {
  it("keeps later selected agents visible in compact mode", () => {
    render(<InlineAgentGrid agents={createAgents()} compact />)

    agentNames.forEach((name) => {
      expect(screen.getByText(name)).toBeInTheDocument()
    })
  })
})

function createAgents(): InlineAgent[] {
  return agentNames.map((name, index) => ({
    name,
    nodeId: `agent-${index + 1}`,
    state: "done",
    status: agentStatus,
    summary: agentSummary,
  }))
}

describe("InlineDispatchFactGrid", () => {
  it("wraps long audit details instead of clamping them", () => {
    render(
      <InlineDispatchFactGrid
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
    expect(detail).not.toHaveClass("line-clamp-2")
  })
})
