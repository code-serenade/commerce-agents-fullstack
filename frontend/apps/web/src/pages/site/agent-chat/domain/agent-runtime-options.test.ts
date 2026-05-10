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
  modelName: "ollama-gpt-oss-20b",
  primaryRole: "data",
  selectedRoles: ["data", "ops"],
}

describe("agent runtime options", () => {
  it("keeps explicit collab role selections for backend metadata", () => {
    expect(runtimeSelectionForStrategy(manualSelection, "collab")).toEqual(
      manualSelection
    )
  })

  it("narrows execution strategy to the primary role", () => {
    expect(runtimeSelectionForStrategy(manualSelection, "execution")).toEqual({
      ...manualSelection,
      collaborationMode: "single",
      selectedRoles: ["data"],
    })
  })

  it("adds a selected role when setting the primary role", () => {
    expect(setRuntimePrimaryRole(manualSelection, "creative")).toEqual({
      ...manualSelection,
      primaryRole: "creative",
      selectedRoles: ["creative", "data", "ops"],
    })
  })

  it("does not remove the manual primary role while other roles remain selected", () => {
    expect(toggleRuntimeRole(manualSelection, "data")).toEqual(manualSelection)
  })

  it("toggles non-primary manual roles", () => {
    expect(toggleRuntimeRole(manualSelection, "ops")).toEqual({
      ...manualSelection,
      selectedRoles: ["data"],
    })
  })
})
