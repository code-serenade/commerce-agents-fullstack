import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getRuntimeNodeStates } from "./swarm-runtime-state-view-model"
import { hasBlockingTaskFailure } from "./thread-boundary-state-view-model"
import { getInlineAgents } from "./thread-inline-view-model"

const t = ((key: string) => {
  if (key === "agentChat.thread.inlineState.done") return "done"
  if (key === "agentChat.thread.agents.data.name") return "data"
  if (key === "agentChat.thread.agents.data.summary") return "data summary"
  if (key === "agentChat.thread.agents.delivery.name") return "delivery"
  if (key === "agentChat.thread.agents.delivery.summary") return "delivery summary"
  return key
}) as never

describe("selected summary boundary wording", () => {
  it("does not treat missing selected summary text as selected product context", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-no-product",
          root_task: [
            textFromCodes([35831, 20998, 26512]),
            textFromCodes([30452, 25773, 38388]),
            textFromCodes([36716, 21270]),
            textFromCodes([26159, 21542]),
            textFromCodes([20581, 24247]),
            textFromCodes([65292]),
            textFromCodes([19981, 24341, 29992, 20219, 20309, 39044, 35774, 21830, 21697]),
          ].join(""),
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: "worker.data",
              title: ["D", "ata"].join(""),
              objective: "live funnel boundary",
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
          graph_id: "graph-no-product",
          node_id: "data",
          task_id: "task-data",
          worker_id: "worker.data",
          role: "data",
          content: "no selected product boundary",
          roleOutput: {
            summary: "no context.selected_summary; use only current user input",
            findings: ["missing live funnel benchmark"],
            recommendations: ["produce category-level framework first"],
            evidence: ["category_level_boundary"],
            risks: ["do not inject default product"],
            openQuestions: [],
          },
          evidence: ["category_level_boundary"],
          risks: ["do not inject default product"],
          open_questions: [],
          status: "failed",
        },
      },
      {
        type: "task_node_failed",
        node_id: "data",
        reason: "worker reported failed status",
      },
      { type: "task_graph_finished", graph_id: "graph-no-product" },
    ]

    expect(hasBlockingTaskFailure(events)).toBe(false)
    expect(getInlineAgents(events, false, t).find((agent) => agent.nodeId === "data")?.state).toBe("boundary")
    expect(getRuntimeNodeStates(events).get("data")?.state).toBe("boundary")
  })
})

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}
