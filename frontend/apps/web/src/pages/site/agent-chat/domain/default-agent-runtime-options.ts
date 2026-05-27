import type { AgentRuntimeOptions } from "./agent-runtime-options"

const runtimeCopyFallback = String()

export const DEFAULT_AGENT_RUNTIME_OPTIONS: AgentRuntimeOptions = {
  collaborationModes: [
    {
      description: runtimeCopyFallback,
      id: "auto",
      isDefault: false,
      label: runtimeCopyFallback,
    },
    {
      description: runtimeCopyFallback,
      id: "single",
      isDefault: false,
      label: runtimeCopyFallback,
    },
    {
      description: runtimeCopyFallback,
      id: "manual",
      isDefault: true,
      label: runtimeCopyFallback,
    },
  ],
  learningLevels: [
    {
      description: runtimeCopyFallback,
      id: "beginner",
      isDefault: false,
      label: runtimeCopyFallback,
    },
    {
      description: runtimeCopyFallback,
      id: "higher_vocational",
      isDefault: true,
      label: runtimeCopyFallback,
    },
    {
      description: runtimeCopyFallback,
      id: "professional",
      isDefault: false,
      label: runtimeCopyFallback,
    },
  ],
  responseModes: [],
  roles: [
    {
      capabilities: ["ops.strategy", "campaign.orchestration"],
      id: "role-ops",
      isDefault: true,
      name: runtimeCopyFallback,
      runtimeRole: "ops",
    },
    {
      capabilities: ["data.analysis", "metrics.diagnosis"],
      id: "role-data",
      isDefault: true,
      name: runtimeCopyFallback,
      runtimeRole: "data",
    },
    {
      capabilities: ["creative.content", "creative.copywriting"],
      id: "role-creative",
      isDefault: true,
      name: runtimeCopyFallback,
      runtimeRole: "creative",
    },
    {
      capabilities: ["service.workflow", "service.sentiment"],
      id: "role-service",
      isDefault: false,
      name: runtimeCopyFallback,
      runtimeRole: "service",
    },
    {
      capabilities: ["ads.optimization", "roi.control"],
      id: "role-douyin-ads",
      isDefault: false,
      name: runtimeCopyFallback,
      runtimeRole: "douyin_ads",
    },
    {
      capabilities: ["finance.profitability", "budget.planning"],
      id: "role-accounting",
      isDefault: false,
      name: runtimeCopyFallback,
      runtimeRole: "accounting",
    },
  ],
  routingModes: [
    {
      description: runtimeCopyFallback,
      id: "commander",
      isDefault: true,
      label: runtimeCopyFallback,
    },
    {
      description: runtimeCopyFallback,
      id: "fast",
      isDefault: false,
      label: runtimeCopyFallback,
    },
  ],
  strategies: [],
}
