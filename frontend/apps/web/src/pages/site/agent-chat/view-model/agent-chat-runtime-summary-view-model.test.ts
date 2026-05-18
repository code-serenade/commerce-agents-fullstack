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
  collaborationModes: [
    {
      id: "auto",
      label: copy("Auto dispatch"),
      description: copy("Default"),
      isDefault: true,
    },
    {
      id: "manual",
      label: copy("Manual team"),
      description: copy("Team roles"),
      isDefault: false,
    },
  ],
  learningLevels: [
    {
      id: "higher_vocational",
      label: copy("Medium"),
      description: copy("Applied"),
      isDefault: true,
    },
    {
      id: "professional",
      label: copy("High"),
      description: copy("Advanced"),
      isDefault: false,
    },
  ],
  models: [],
  responseModes: [],
  roles: [
    {
      id: "role-ops",
      name: copy("Ops"),
      runtimeRole: "ops",
      capabilities: [],
      isDefault: false,
    },
    {
      id: "role-data",
      name: copy("Data"),
      runtimeRole: "data",
      capabilities: [],
      isDefault: false,
    },
    {
      id: "role-creative",
      name: copy("Creative"),
      runtimeRole: "creative",
      capabilities: [],
      isDefault: false,
    },
  ],
  strategies: [],
}

function copy(value: string) {
  return value
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
