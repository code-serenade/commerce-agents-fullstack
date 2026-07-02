import type { AgentRuntimeOptions } from "./agent-runtime-options"

const runtimeCopyFallback = String()

export const DEFAULT_AGENT_RUNTIME_OPTIONS: AgentRuntimeOptions = {
  collaborationModes: [
    {
      description: runtimeCopyFallback,
      id: "auto",
      isDefault: true,
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
      isDefault: false,
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
      capabilities: ["search.evidence", "source.verification"],
      id: "role-search",
      isDefault: false,
      name: runtimeCopyFallback,
      runtimeRole: "search",
    },
    {
      capabilities: ["data.analysis", "metrics.diagnosis"],
      id: "role-data",
      isDefault: true,
      name: runtimeCopyFallback,
      runtimeRole: "data",
    },
    {
      capabilities: ["service.workflow", "service.sentiment"],
      id: "role-service",
      isDefault: false,
      name: runtimeCopyFallback,
      runtimeRole: "service",
    },
    {
      capabilities: ["creative.content", "creative.copywriting"],
      id: "role-creative",
      isDefault: true,
      name: runtimeCopyFallback,
      runtimeRole: "creative",
    },
    {
      capabilities: ["engineering.architecture", "engineering.reliability"],
      id: "role-engineering",
      isDefault: false,
      name: runtimeCopyFallback,
      runtimeRole: "engineering",
    },
    {
      capabilities: ["finance.profitability", "budget.planning"],
      id: "role-accounting",
      isDefault: false,
      name: runtimeCopyFallback,
      runtimeRole: "accounting",
    },
    {
      capabilities: ["design.visual", "design.layout"],
      id: "role-design",
      isDefault: false,
      name: runtimeCopyFallback,
      runtimeRole: "design",
    },
    {
      capabilities: ["seo.optimization", "web.content"],
      id: "role-web",
      isDefault: false,
      name: runtimeCopyFallback,
      runtimeRole: "web",
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
