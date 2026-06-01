import { describe, expect, it } from "vitest"
import {
  defaultRuntimeSelection,
  restoreRuntimeSelection,
  runtimeSelectionForStrategy,
  setRuntimeCollaborationMode,
  setRuntimePrimaryRole,
  toggleRuntimeRole,
  type AgentChatRuntimeSelection,
} from "./agent-runtime-options"

const manualSelection: AgentChatRuntimeSelection = {
  collaborationMode: "manual",
  learningLevel: "higher_vocational",
  primaryRole: "data",
  routingMode: "commander",
  selectedRoles: ["data", "ops"],
}
const manualCopy = "Manual"
const autoCopy = "Auto"
const higherVocationalCopy = "Higher vocational"

describe("agent runtime options", () => {
  it("uses the same default answer setup before backend options finish loading", () => {
    expect(defaultRuntimeSelection(null)).toMatchObject({
      collaborationMode: "auto",
      learningLevel: "higher_vocational",
      primaryRole: "ops",
      selectedRoles: [],
    })
  })

  it("defaults new commerce chats to backend-dispatched role selection when available", () => {
    expect(
      defaultRuntimeSelection({
        collaborationModes: [
          {
            id: "auto",
            label: autoCopy,
            description: autoCopy,
            isDefault: true,
          },
          {
            id: "manual",
            label: manualCopy,
            description: manualCopy,
            isDefault: false,
          },
        ],
        learningLevels: [
          {
            id: "higher_vocational",
            label: higherVocationalCopy,
            description: higherVocationalCopy,
            isDefault: true,
          },
        ],
        responseModes: [],
        routingModes: [],
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
      primaryRole: "ops",
      routingMode: "commander",
      selectedRoles: [],
    })
  })

  it("keeps explicit multi-role selections for backend metadata", () => {
    expect(runtimeSelectionForStrategy(manualSelection)).toEqual(manualSelection)
  })

  it("shows ops as primary when chat is selected with specialist roles", () => {
    const restored = {
      ...manualSelection,
      primaryRole: "chat",
      selectedRoles: ["chat", "ops", "data", "service"],
    }
    expect(runtimeSelectionForStrategy(restored)).toEqual({
      ...restored,
      collaborationMode: "manual",
      primaryRole: "ops",
      selectedRoles: ["ops", "data", "service"],
    })
  })

  it("keeps explicit role mode for professional answers", () => {
    expect(runtimeSelectionForStrategy(manualSelection)).toEqual({
      ...manualSelection,
    })
  })

  it("keeps auto collaboration as backend-dispatched role selection", () => {
    expect(
      runtimeSelectionForStrategy({
        ...manualSelection,
        collaborationMode: "auto",
        primaryRole: "chat",
        selectedRoles: [],
      })
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

  it("preserves explicit manual support roles when changing the primary role", () => {
    expect(
      setRuntimePrimaryRole(
        {
          ...manualSelection,
          primaryRole: "data",
          selectedRoles: ["data", "service", "accounting", "engineering"],
        },
        "search"
      )
    ).toEqual({
      ...manualSelection,
      primaryRole: "search",
      selectedRoles: ["search", "data", "service", "accounting", "engineering"],
    })
  })

  it("keeps one selected role in single-role assignment", () => {
    const single = setRuntimeCollaborationMode(manualSelection, "single")

    expect(single).toEqual({
      ...manualSelection,
      collaborationMode: "single",
      primaryRole: "data",
      selectedRoles: ["data"],
    })
    expect(setRuntimePrimaryRole(single, "creative")).toEqual({
      ...single,
      primaryRole: "creative",
      selectedRoles: ["creative"],
    })
  })

  it("keeps at least two selected roles in manual multi-role assignment", () => {
    expect(
      setRuntimeCollaborationMode(
        {
          ...manualSelection,
          selectedRoles: ["data"],
        },
        "manual"
      )
    ).toEqual({
      ...manualSelection,
      collaborationMode: "manual",
      primaryRole: "data",
      selectedRoles: ["data", "ops", "creative"],
    })
  })

  it("starts manual multi-role mode with a clear three-role team from auto defaults", () => {
    expect(
      setRuntimeCollaborationMode(
        {
          ...manualSelection,
          collaborationMode: "auto",
          primaryRole: "ops",
          selectedRoles: [],
        },
        "manual"
      )
    ).toEqual({
      ...manualSelection,
      collaborationMode: "manual",
      primaryRole: "ops",
      selectedRoles: ["ops", "data", "creative"],
    })
  })

  it("keeps primary and last support selected in manual multi-role mode", () => {
    expect(toggleRuntimeRole(manualSelection, "data")).toEqual(manualSelection)
    expect(toggleRuntimeRole(manualSelection, "ops")).toEqual(manualSelection)
  })

  it("can remove one support role while another support role remains selected", () => {
    expect(
      toggleRuntimeRole(
        {
          ...manualSelection,
          selectedRoles: ["data", "ops", "creative"],
        },
        "ops"
      )
    ).toEqual({
      ...manualSelection,
      selectedRoles: ["data", "creative"],
    })
  })

  it("keeps ops as the primary when adding support roles after chat", () => {
    expect(
      toggleRuntimeRole(
        {
          ...manualSelection,
          primaryRole: "ops",
          selectedRoles: ["chat", "ops", "data", "creative"],
        },
        "engineering"
      )
    ).toEqual({
      ...manualSelection,
      primaryRole: "ops",
      selectedRoles: ["ops", "chat", "data", "creative", "engineering"],
    })
  })

  it("restores persisted runtime selection through available backend options", () => {
    expect(
      restoreRuntimeSelection(
        manualSelection,
        {
          collaborationMode: "manual",
          learningLevel: "higher_vocational",
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
          learningLevels: [
            {
              id: "higher_vocational",
              label: higherVocationalCopy,
              description: higherVocationalCopy,
              isDefault: true,
            },
          ],
          responseModes: [],
          routingModes: [],
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
      primaryRole: "creative",
      routingMode: "commander",
      selectedRoles: ["creative"],
    })
  })
})
