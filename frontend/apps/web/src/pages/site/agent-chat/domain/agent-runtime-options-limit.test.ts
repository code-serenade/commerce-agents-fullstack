import { describe, expect, it } from "vitest"
import {
  runtimeSelectionForStrategy,
  setRuntimePrimaryRole,
  toggleRuntimeRole,
  type AgentChatRuntimeSelection,
} from "./agent-runtime-options"

const manualSelection: AgentChatRuntimeSelection = {
  collaborationMode: "manual",
  learningLevel: "higher_vocational",
  primaryRole: "ops",
  routingMode: "commander",
  selectedRoles: ["ops", "data"],
}

describe("agent runtime option role limits", () => {
  it("keeps manual selected roles inside the backend SSE limit when changing primary role", () => {
    expect(
      setRuntimePrimaryRole(
        {
          ...manualSelection,
          selectedRoles: [
            "ops",
            "data",
            "creative",
            "service",
            "engineering",
            "accounting",
            "search",
            "web",
          ],
        },
        "ads"
      )
    ).toEqual({
      ...manualSelection,
      primaryRole: "ads",
      selectedRoles: [
        "ads",
        "ops",
        "data",
        "creative",
        "service",
        "engineering",
        "accounting",
        "search",
      ],
    })
  })

  it("ignores the ninth manual support role so SSE metadata stays valid", () => {
    expect(
      toggleRuntimeRole(
        {
          ...manualSelection,
          selectedRoles: [
            "ops",
            "data",
            "creative",
            "service",
            "engineering",
            "accounting",
            "search",
            "web",
          ],
        },
        "ads"
      )
    ).toEqual({
      ...manualSelection,
      selectedRoles: [
        "ops",
        "data",
        "creative",
        "service",
        "engineering",
        "accounting",
        "search",
        "web",
      ],
    })
  })

  it("normalizes restored selected roles to the backend SSE limit", () => {
    expect(
      runtimeSelectionForStrategy({
        ...manualSelection,
        selectedRoles: [
          "ops",
          "data",
          "creative",
          "service",
          "engineering",
          "accounting",
          "search",
          "web",
          "ads",
        ],
      }).selectedRoles
    ).toEqual([
      "ops",
      "data",
      "creative",
      "service",
      "engineering",
      "accounting",
      "search",
      "web",
    ])
  })
})
