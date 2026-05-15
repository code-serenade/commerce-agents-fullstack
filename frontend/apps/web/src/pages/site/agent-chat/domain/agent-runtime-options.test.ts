import { describe, expect, it } from "vitest"
import {
  restoreRuntimeSelection,
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
const manualCopy = "Manual"
const higherVocationalCopy = "Higher vocational"

describe("agent runtime options", () => {
  it("keeps explicit collab role selections for backend metadata", () => {
    expect(runtimeSelectionForStrategy(manualSelection, "collab")).toEqual(
      manualSelection
    )
  })

  it("uses the chat role for quick execution answers", () => {
    expect(runtimeSelectionForStrategy(manualSelection, "execution")).toEqual({
      ...manualSelection,
      collaborationMode: "single",
      primaryRole: "chat",
      selectedRoles: ["chat"],
    })
  })

  it("uses the chat role for guided learning answers", () => {
    expect(runtimeSelectionForStrategy(manualSelection, "learning")).toEqual({
      ...manualSelection,
      collaborationMode: "single",
      primaryRole: "chat",
      selectedRoles: ["chat"],
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

  it("restores persisted runtime selection through available backend options", () => {
    expect(
      restoreRuntimeSelection(
        manualSelection,
        {
          collaborationMode: "manual",
          learningLevel: "higher_vocational",
          modelName: "ollama-gpt-oss-20b",
          primaryRole: "creative",
          selectedRoles: ["creative", "missing"],
        },
        {
          collaborationModes: [
            {
              id: "manual",
              label: manualCopy,
              description: manualCopy,
              isDefault: true,
            },
          ],
          defaultModelName: "ollama-gpt-oss-20b",
          learningLevels: [
            {
              id: "higher_vocational",
              label: higherVocationalCopy,
              description: higherVocationalCopy,
              isDefault: true,
            },
          ],
          models: [
            {
              name: "ollama-gpt-oss-20b",
              model: "gpt-oss:20b",
              isDefault: true,
            },
          ],
          responseModes: [],
          roles: [
            {
              id: "role-data",
              name: "Data",
              runtimeRole: "data",
              capabilities: [],
              isDefault: true,
            },
            {
              id: "role-creative",
              name: "Creative",
              runtimeRole: "creative",
              capabilities: [],
              isDefault: false,
            },
          ],
          strategies: [],
        }
      )
    ).toEqual({
      collaborationMode: "manual",
      learningLevel: "higher_vocational",
      modelName: "ollama-gpt-oss-20b",
      primaryRole: "creative",
      selectedRoles: ["creative"],
    })
  })
})
