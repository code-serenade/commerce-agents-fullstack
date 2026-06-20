import { describe, expect, it } from "vitest"
import type { TFunction } from "i18next"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import { DEFAULT_AGENT_RUNTIME_OPTIONS } from "../domain/default-agent-runtime-options"
import {
  buildCompactRuntimeSummary,
  selectedRoleSummary,
} from "./agent-chat-runtime-summary-view-model"

const t = ((key: string, values?: Record<string, unknown>) => {
  const copy: Record<string, string> = {
    "agentChat.strategy.options.learning.shortHint": "Teaching",
    "agentChat.strategy.options.quick.shortHint": "Short",
    "agentChat.strategy.options.execution.shortHint": "Full",
    "agentChat.composer.runtime.learningLevels.higher_vocational.label":
      "Applied",
    "agentChat.composer.runtime.roleLabels.ops.name": "Ops",
    "agentChat.composer.runtime.roleModes.manual.label": "Multi-agent",
    "agentChat.composer.runtime.routingModes.commander.label":
      "Training route",
    "agentChat.composer.settings.autoRolesSummary": "Backend auto",
    "agentChat.composer.settings.multiAgentSummary": `${values?.count ?? 0} references`,
    "agentChat.composer.settings.rolesCount": `${values?.count ?? 0} references`,
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
  responseModes: [],
  routingModes: [
    {
      id: "commander",
      label: copy("Commander"),
      description: copy("Deep route"),
      isDefault: true,
    },
  ],
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
      primaryRole: "ops",
      routingMode: "commander",
      selectedRoles: [],
    }

    expect(
      buildCompactRuntimeSummary({
        runtimeOptions,
        runtimeSelection: selection,
        strategyId: "learning",
        t,
      })
    ).toBe("Teaching / Training route / Auto dispatch / Applied / Backend auto")
  })

  it("shows agent collaboration count for manual multi-agent mode", () => {
    const selection: AgentChatRuntimeSelection = {
      collaborationMode: "manual",
      learningLevel: "professional",
      primaryRole: "data",
      routingMode: "commander",
      selectedRoles: ["data", "ops", "creative"],
    }

    expect(
      selectedRoleSummary({
        runtimeOptions,
        runtimeSelection: selection,
        t,
      })
    ).toBe("3 references")
  })

  it("uses locale copy for fallback runtime options without showing internal ids", () => {
    const selection: AgentChatRuntimeSelection = {
      collaborationMode: "manual",
      learningLevel: "higher_vocational",
      primaryRole: "ops",
      routingMode: "commander",
      selectedRoles: ["ops", "data", "creative"],
    }

    expect(
      buildCompactRuntimeSummary({
        runtimeOptions: DEFAULT_AGENT_RUNTIME_OPTIONS,
        runtimeSelection: selection,
        strategyId: "learning",
        t,
      })
    ).toBe("Teaching / Training route / Multi-agent / Applied / 3 references")
  })
})
