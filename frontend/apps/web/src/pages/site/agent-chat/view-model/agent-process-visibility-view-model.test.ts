import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentChatMessage } from "../types"
import {
  findLatestProcessTarget,
  getProcessRuntimeEvents,
  shouldRenderMessageProcess,
} from "./agent-process-visibility-view-model"

const opsTitle = ["Operations", "plan"].join(" ")
const opsObjective = ["Build", "execution plan"].join(" ")
const directChatTitle = ["Direct", "chat"].join(" ")
const directChatObjective = ["Answer", "directly"].join(" ")

const liveEvents: AgentRuntimeEvent[] = [
  {
    type: "task_graph_planned",
    graph: {
      graph_id: "graph-1",
      root_task: "coffee marketing task",
      nodes: [
        {
          id: "ops",
          role: "ops",
          worker_id: "worker.ops",
          title: opsTitle,
          objective: opsObjective,
          input_refs: [],
          acceptance: [],
          max_attempts: 1,
        },
      ],
    },
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
    const oldEvents = liveEvents
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

  it("does not expose the cluster process panel for direct chat routes", () => {
    const directEvents: AgentRuntimeEvent[] = [
      {
        type: "route_decided",
        route_mode: "chat_bypass",
        primary_runtime_role: "chat",
        support_runtime_roles: [],
      },
    ]
    const messages: AgentChatMessage[] = [
      { id: "user-1", role: "user", content: "short question" },
      { id: "assistant-1", role: "assistant", content: "short answer" },
    ]

    expect(shouldRenderMessageProcess(messages[1]!, directEvents)).toBe(false)
    expect(findLatestProcessTarget(messages, directEvents)).toBeNull()
  })

  it("does not expose the cluster process panel for a single chat task node", () => {
    const chatNodeEvents: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-chat",
          root_task: "short question",
          nodes: [
            {
              id: "chat",
              role: "chat",
              worker_id: "worker.chat",
              title: directChatTitle,
              objective: directChatObjective,
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
          ],
        },
      },
    ]
    const messages: AgentChatMessage[] = [
      { id: "user-1", role: "user", content: "short question" },
      { id: "assistant-1", role: "assistant", content: "short answer" },
    ]

    expect(shouldRenderMessageProcess(messages[1]!, chatNodeEvents)).toBe(false)
    expect(findLatestProcessTarget(messages, chatNodeEvents)).toBeNull()
  })
})
