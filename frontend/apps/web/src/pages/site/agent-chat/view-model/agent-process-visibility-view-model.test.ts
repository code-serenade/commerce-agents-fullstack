import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentChatMessage } from "../types"
import {
  findLatestProcessTarget,
  getProcessRuntimeEvents,
  shouldRenderMessageProcess,
} from "./agent-process-visibility-view-model"

const liveEvents: AgentRuntimeEvent[] = [
  {
    type: "route_decided",
    route_mode: "task_graph",
    primary_runtime_role: "ops",
    support_runtime_roles: ["data", "creative"],
  },
]

describe("agent process visibility", () => {
  it("uses live runtime events for the latest assistant turn before persistence catches up", () => {
    const messages: AgentChatMessage[] = [
      { id: "welcome", role: "assistant", content: "welcome" },
      { id: "user-1", role: "user", content: "coffee marketing task" },
      { id: "assistant-1", role: "assistant", content: "", status: "streaming" },
    ]

    const processEvents = getProcessRuntimeEvents(messages, 2, liveEvents)

    expect(processEvents).toBe(liveEvents)
    expect(shouldRenderMessageProcess(messages[2]!, processEvents)).toBe(true)
    expect(findLatestProcessTarget(messages, liveEvents)).toEqual({
      anchorLabel: "#01",
      messageId: "assistant-1",
      task: "coffee marketing task",
    })
  })

  it("keeps older assistant turns bound to their own persisted runtime events", () => {
    const oldEvents: AgentRuntimeEvent[] = [{ type: "finished" }]
    const messages: AgentChatMessage[] = [
      { id: "user-1", role: "user", content: "first question" },
      {
        id: "assistant-1",
        role: "assistant",
        content: "first answer",
        runtimeEvents: oldEvents,
      },
      { id: "user-2", role: "user", content: "second question" },
      { id: "assistant-2", role: "assistant", content: "second answer" },
    ]

    expect(getProcessRuntimeEvents(messages, 1, liveEvents)).toBe(oldEvents)
    expect(getProcessRuntimeEvents(messages, 3, liveEvents)).toBe(liveEvents)
  })
})
