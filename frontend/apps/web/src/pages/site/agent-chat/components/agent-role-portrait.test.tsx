import { render } from "@testing-library/react"
import { describe, expect, it } from "vitest"
import type { AgentRoleActivity } from "../view-model/agent-role-cluster-view-model"
import { AgentRolePortrait } from "./agent-role-portrait"

describe("AgentRolePortrait", () => {
  it("marks the role portrait as working while the role is active", () => {
    const { rerender } = render(
      <AgentRolePortrait
        activity={createActivity("working")}
        animate
      />
    )

    const workingPortrait = document.querySelector(
      "[data-agent-portrait-asset='ops-portrait']"
    )

    expect(workingPortrait).toHaveAttribute(
      "data-agent-portrait-source",
      "working"
    )
    expect(workingPortrait).toHaveClass("agent-role-portrait__sprite--working")

    rerender(<AgentRolePortrait activity={createActivity("done")} />)

    const donePortrait = document.querySelector(
      "[data-agent-portrait-asset='ops-portrait']"
    )

    expect(donePortrait).toHaveAttribute("data-agent-portrait-source", "static")
    expect(donePortrait).not.toHaveClass("agent-role-portrait__sprite--working")
  })

  it("also marks a reworking role as working", () => {
    render(
      <AgentRolePortrait
        activity={createActivity("reworking")}
        animate
      />
    )

    const portrait = document.querySelector(
      "[data-agent-portrait-asset='ops-portrait']"
    )

    expect(portrait).toHaveAttribute("data-agent-portrait-source", "working")
    expect(portrait).toHaveClass("agent-role-portrait__sprite--working")
  })

  it("keeps active showcase portraits static when animation is not requested", () => {
    render(<AgentRolePortrait activity={createActivity("working")} />)

    const portrait = document.querySelector(
      "[data-agent-portrait-asset='ops-portrait']"
    )

    expect(portrait).toHaveAttribute("data-agent-portrait-source", "static")
    expect(portrait).not.toHaveClass("agent-role-portrait__sprite--working")
  })

  it("keeps the portrait frame rectangular so source role art is not cropped", () => {
    render(<AgentRolePortrait activity={createActivity("working")} />)

    const frame = document.querySelector("[data-agent-portrait-frame='rect']")
    const surface = document.querySelector(
      "[data-agent-portrait-frame-surface='rect']"
    )

    expect(frame).toHaveClass("size-20")
    expect(surface).toHaveClass("rounded-md")
    expect(surface?.className).not.toContain("rounded-full")
  })
})

function createActivity(
  state: AgentRoleActivity["state"]
): AgentRoleActivity {
  return {
    action: "Checking the task",
    agent: {
      name: "Operations strategist",
      nodeId: "ops",
      state,
      status: state,
      summary: "Plan the next action",
    },
    contribution: "Turns findings into execution priorities",
    lane: 0,
    role: "ops",
    state,
  }
}
