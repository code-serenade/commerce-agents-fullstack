import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getRuntimeNodeStates } from "./swarm-runtime-state-view-model"
import {
  getInlineAgents,
  getInlineProcessStage,
} from "./thread-inline-view-model"

const t = ((key: string) => {
  if (key === "agentChat.thread.processStage.finished") return "finished"
  if (key === "agentChat.thread.processStage.failed") return "failed"
  if (key === "agentChat.thread.inlineState.data.name") return "data"
  if (key === "agentChat.thread.inlineState.data.summary") return "data summary"
  if (key === "agentChat.thread.agents.data.name") return "data"
  if (key === "agentChat.thread.agents.data.summary") return "data summary"
  return key
}) as never

const coffeeCategoryTask = [
  textFromCodes([35831, 20998, 26512]),
  textFromCodes([21654, 21857, 31867]),
  textFromCodes([21830, 21697]),
].join("")

const productIdentityPrompt = [
  textFromCodes([35831, 25552, 20379]),
  textFromCodes([21654, 21857, 31867, 21830, 21697]),
  textFromCodes([30340, 20135, 21697, 32534, 21495]),
  textFromCodes([25110, 21517, 31216]),
  textFromCodes([20197, 20415, 26597, 35810]),
  textFromCodes([30456, 20851, 19994, 21153, 25351, 26631]),
].join("")

describe("category boundary evaluation display", () => {
  it("keeps reported failed category role reviews as boundary when evaluation later passes", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: coffeeCategoryTask,
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: "worker.data",
              title: ["Data", "diagnosis"].join(" "),
              objective: "category-level data diagnosis",
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
          ],
        },
      },
      {
        type: "task_node_reported",
        node_id: "data",
        report: {
          graph_id: "graph-1",
          node_id: "data",
          task_id: "task-1",
          worker_id: "worker.data",
          role: "data",
          content: productIdentityPrompt,
          role_output: {
            summary: productIdentityPrompt,
            findings: ["missing real funnel metrics"],
            recommendations: [],
            evidence: ["no context.selected_summary"],
            risks: [],
            open_questions: [],
          },
          evidence: ["no context.selected_summary"],
          risks: [],
          open_questions: [],
          status: "failed",
        },
      },
      {
        type: "task_node_evaluated",
        evaluation: {
          node_id: "data",
          passed: true,
          score: 1,
          reasons: ["data role should provide at least two findings"],
          rework_instruction: null,
        },
      },
      { type: "task_node_passed", node_id: "data" },
      { type: "task_graph_finished", graph_id: "graph-1" },
    ]

    expect(
      getInlineAgents(events, false, t).find((agent) => agent.nodeId === "data")
        ?.state
    ).toBe("boundary")
    const snapshot = getRuntimeNodeStates(events).get("data")
    expect(snapshot?.state).toBe("boundary")
    expect(snapshot?.evaluationReasons).toEqual([])
    expect(
      getInlineProcessStage(
        { hasRuntime: true, isStreaming: false, runtimeEvents: events },
        t
      )
    ).toBe("finished")
  })
})

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}
