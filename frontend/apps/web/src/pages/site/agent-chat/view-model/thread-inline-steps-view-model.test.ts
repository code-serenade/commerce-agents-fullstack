import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getInlineSteps } from "./thread-inline-view-model"

const t = ((key: string) => {
  if (key === "agentChat.thread.steps.assignTask") return "assign task"
  if (key === "agentChat.thread.steps.chatOnlyRoute") return "direct answer"
  if (key === "agentChat.thread.steps.directRoute") return "direct route"
  if (key === "agentChat.thread.steps.directSavedToReport") {
    return "direct saved"
  }
  if (key === "agentChat.thread.steps.identifyNeed") return "identify need"
  if (key === "agentChat.thread.steps.readTarget") return "read target"
  if (key === "agentChat.thread.steps.syncResult") return "sync result"
  if (key === "agentChat.thread.roleFallbackTitles.data") {
    return "Data diagnosis"
  }
  return key
}) as never
const dataTitle = ["Data", " diagnosis"].join("")
const dataObjective = ["Check metric scope", "and missing inputs first"].join(" ")
const opsTitle = ["Operations", "plan"].join(" ")
const opsObjective = ["Integrate upstream roles", "into an execution plan"].join(" ")

describe("getInlineSteps", () => {
  it("uses backend task graph nodes instead of a fixed three-step template", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-steps",
          root_task: "multi-role marketing plan",
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: "worker.data",
              title: dataTitle,
              objective: dataObjective,
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
            {
              id: "ops",
              role: "ops",
              worker_id: "worker.ops",
              title: opsTitle,
              objective: opsObjective,
              input_refs: ["data"],
              acceptance: [],
              max_attempts: 1,
            },
          ],
        },
      },
    ]

    expect(getInlineSteps(events, false, t)).toEqual([
      [dataTitle, dataObjective],
      [opsTitle, opsObjective],
    ])
  })

  it("keeps need detection visible for direct-answer runs", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "route_decided",
        route_mode: "chat_bypass",
        primary_runtime_role: "chat",
        support_runtime_roles: [],
      },
    ]

    expect(getInlineSteps(events, false, t)).toEqual([
      ["identify need", "direct route"],
      ["assign task", "direct answer"],
      ["sync result", "direct saved"],
    ])
  })
})
