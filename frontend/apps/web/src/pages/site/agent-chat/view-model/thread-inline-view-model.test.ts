import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { isChatOnlyRuntime } from "./thread-inline-view-model"

describe("isChatOnlyRuntime", () => {
  it("treats chat_bypass route as direct chat without swarm UI", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "route_decided",
        route_mode: "chat_bypass",
        primary_runtime_role: "chat",
        support_runtime_roles: [],
      },
    ]

    expect(isChatOnlyRuntime(events)).toBe(true)
  })

  it("keeps task graph route visible for multi-role work", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "route_decided",
        route_mode: "task_graph",
        primary_runtime_role: "ops",
        support_runtime_roles: ["data", "creative"],
      },
    ]

    expect(isChatOnlyRuntime(events)).toBe(false)
  })

  it("does not hide task graph UI only because the primary role is chat", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "route_decided",
        route_mode: "task_graph",
        primary_runtime_role: "chat",
        support_runtime_roles: [],
      },
    ]

    expect(isChatOnlyRuntime(events)).toBe(false)
  })
})
