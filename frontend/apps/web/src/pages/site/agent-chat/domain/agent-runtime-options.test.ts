import { describe, expect, it } from "vitest"
import {
  defaultRuntimeSelection,
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
const autoCopy = "Auto"
const higherVocationalCopy = "Higher vocational"

describe("agent runtime options", () => {
  it("defaults new commerce chats to backend auto role dispatch when available", () => {
    expect(
      defaultRuntimeSelection({
        collaborationModes: [
          {
            id: "auto",
            label: autoCopy,
            description: autoCopy,
            isDefault: false,
          },
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
            id: "role-engineering",
            name: "Engineering",
            runtimeRole: "engineering",
            capabilities: [],
            isDefault: false,
          },
          {
            id: "role-data",
            name: "Data",
            runtimeRole: "data",
            capabilities: [],
            isDefault: true,
          },
          {
            id: "role-ops",
            name: "Ops",
            runtimeRole: "ops",
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
          {
            id: "role-service",
            name: "Service",
            runtimeRole: "service",
            capabilities: [],
            isDefault: false,
          },
        ],
        strategies: [],
      })
    ).toEqual({
      collaborationMode: "auto",
      learningLevel: "higher_vocational",
      modelName: "ollama-gpt-oss-20b",
      primaryRole: "ops",
      selectedRoles: ["ops", "data", "creative", "service", "engineering"],
    })
  })

  it("keeps explicit collab role selections for backend metadata", () => {
    expect(runtimeSelectionForStrategy(manualSelection, "collab")).toEqual(
      manualSelection
    )
  })

  it("shows ops as collab primary when chat is selected with specialist roles", () => {
    const restored = {
      ...manualSelection,
      primaryRole: "chat",
      selectedRoles: ["chat", "ops", "data", "service"],
    }
    expect(runtimeSelectionForStrategy(restored, "collab")).toEqual({
      ...restored,
      collaborationMode: "manual",
      primaryRole: "ops",
      selectedRoles: ["ops", "data", "service"],
    })
  })

  it("keeps explicit role mode for professional answers", () => {
    expect(runtimeSelectionForStrategy(manualSelection, "execution")).toEqual({
      ...manualSelection,
    })
  })

  it("keeps explicit role mode for guided learning answers and lowers explanation level", () => {
    expect(runtimeSelectionForStrategy(manualSelection, "learning")).toEqual({
      ...manualSelection,
      learningLevel: "beginner",
    })
  })

  it("keeps auto collaboration as backend-dispatched role selection", () => {
    expect(
      runtimeSelectionForStrategy(
        {
          ...manualSelection,
          collaborationMode: "auto",
          primaryRole: "chat",
          selectedRoles: [],
        },
        "collab"
      )
    ).toEqual({
      ...manualSelection,
      collaborationMode: "auto",
      primaryRole: "ops",
      selectedRoles: [],
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

  it("keeps ops as the collab primary when adding support roles after chat", () => {
    expect(
      toggleRuntimeRole(
        {
          ...manualSelection,
          primaryRole: "chat",
          selectedRoles: ["chat", "ops", "data", "creative"],
        },
        "engineering"
      )
    ).toEqual({
      ...manualSelection,
      primaryRole: "ops",
      selectedRoles: ["chat", "ops", "data", "creative", "engineering"],
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
