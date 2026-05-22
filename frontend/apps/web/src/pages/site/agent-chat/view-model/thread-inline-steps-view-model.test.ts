import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getInlineSteps } from "./thread-inline-view-model"

const t = ((key: string) => {
  if (key === "agentChat.thread.steps.answerReady") return "answer ready"
  if (key === "agentChat.thread.steps.answerSynced") return "answer synced"
  if (key === "agentChat.thread.steps.assignTask") return "assign task"
  if (key === "agentChat.thread.steps.dispatchJudge") return "dispatch judge"
  if (key === "agentChat.thread.steps.dispatchRoute") return "chat_bypass direct answer"
  if (key === "agentChat.thread.steps.noExtraSkill") return "no extra skill"
  if (key === "agentChat.thread.steps.readTarget") return "read target"
  if (key === "agentChat.thread.steps.runtimeAction") return "runtime action"
  if (key === "agentChat.thread.steps.targetRules") return "target rules"
  if (key === "agentChat.thread.roleFallbackTitles.data") {
    return "Data diagnosis"
  }
  if (key === "agentChat.thread.roleFallbackTitles.chat") {
    return "direct answer"
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

  it("keeps direct-answer runs as light dispatch steps instead of fake cluster steps", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "route_decided",
        route_mode: "chat_bypass",
        primary_runtime_role: "chat",
        support_runtime_roles: [],
      },
    ]

    expect(getInlineSteps(events, false, t)).toEqual([
      ["read target", "target rules"],
      ["dispatch judge", "chat_bypass direct answer"],
      ["runtime action", "no extra skill"],
      ["answer ready", "answer synced"],
    ])
  })
})
