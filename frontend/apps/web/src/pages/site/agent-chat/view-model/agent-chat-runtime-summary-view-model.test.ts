import { describe, expect, it } from "vitest"
import type { TFunction } from "i18next"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import {
  buildCompactRuntimeSummary,
  selectedRoleSummary,
  shortModelName,
} from "./agent-chat-runtime-summary-view-model"

const t = ((key: string, values?: Record<string, unknown>) => {
  const copy: Record<string, string> = {
    "agentChat.strategy.options.learning.shortHint": "Default",
    "agentChat.strategy.options.quick.shortHint": "Short",
    "agentChat.strategy.options.execution.shortHint": "Full",
    "agentChat.composer.settings.autoRolesSummary": "Backend auto",
    "agentChat.composer.settings.rolesCount": `${values?.count ?? 0} roles`,
  }
  return copy[key] ?? key
}) as TFunction

const runtimeOptions: AgentRuntimeOptions = {
  collaborationModes: [],
  learningLevels: [],
  models: [],
  responseModes: [],
  roles: [
    {
      id: "role-ops",
      name: "Ops",
      runtimeRole: "ops",
      capabilities: [],
      isDefault: false,
    },
    {
      id: "role-data",
      name: "Data",
      runtimeRole: "data",
      capabilities: [],
      isDefault: false,
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

describe("agent chat runtime summary view model", () => {
  it("keeps the visible runtime summary short and readable", () => {
    const selection: AgentChatRuntimeSelection = {
      collaborationMode: "auto",
      learningLevel: "higher_vocational",
      modelName: "doubao-pro-32k",
      primaryRole: "ops",
      selectedRoles: [],
    }

    expect(
      buildCompactRuntimeSummary({
        depthLabel: "Medium",
        roleModeLabel: "Auto dispatch",
        runtimeOptions,
        runtimeSelection: selection,
        strategyId: "learning",
        t,
      })
    ).toBe("Default / Auto dispatch / Medium / Doubao / Backend auto")
  })

  it("shows the lead role and support count for manual multi-role mode", () => {
    const selection: AgentChatRuntimeSelection = {
      collaborationMode: "manual",
      learningLevel: "professional",
      modelName: "llm.commerce-model",
      primaryRole: "data",
      selectedRoles: ["data", "ops", "creative"],
    }

    expect(
      selectedRoleSummary({
        runtimeOptions,
        runtimeSelection: selection,
        t,
      })
    ).toBe("Data+2")
  })

  it("normalizes long model ids without hiding non-Doubao names", () => {
    expect(shortModelName("doubao-pro-32k")).toBe("Doubao")
    expect(shortModelName("llm.commerce-model")).toBe("commerce")
  })
})
