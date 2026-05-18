import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import {
  getInlineAgents,
  getInlineProcessStage,
} from "./thread-inline-view-model"
import { getRuntimeNodeStates } from "./swarm-runtime-state-view-model"

const t = ((key: string) => {
  if (key === "agentChat.thread.processStage.finished") return "finished"
  if (key === "agentChat.thread.processStage.failed") return "failed"
  if (key === "agentChat.thread.inlineState.boundary") return "boundary"
  if (key === "agentChat.thread.inlineState.queued") return "queued"
  if (key === "agentChat.thread.agents.data.name") return "data"
  if (key === "agentChat.thread.agents.data.summary") return "data summary"
  if (key === "agentChat.thread.agents.delivery.name") return "delivery"
  if (key === "agentChat.thread.agents.delivery.summary") return "delivery summary"
  return key
}) as never

describe("legacy inline boundary events", () => {
  it("treats legacy role contract failures as reviewable category boundaries", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: textFromCodes([35831, 20998, 26512, 21654, 21857, 31867, 21830, 21697]),
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: "worker.data",
              title: ["Data", " diagnosis"].join(""),
              objective: "category-level data diagnosis",
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
            {
              id: "service",
              role: "service",
              worker_id: "worker.service",
              title: ["Service", "support"].join(" "),
              objective: "category-level service boundary",
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
          ],
        },
      },
      {
        type: "task_node_failed",
        node_id: "data",
        reason: "data role should provide at least two findings",
      },
      {
        type: "task_node_failed",
        node_id: "service",
        reason: "service role output lacks service-specific content",
      },
      { type: "task_graph_finished", graph_id: "graph-1" },
    ]

    expect(
      getInlineProcessStage({
        hasRuntime: true,
        isStreaming: false,
        runtimeEvents: events,
      }, t)
    ).toBe("finished")
    expect(getInlineAgents(events, false, t).find((agent) => agent.nodeId === "data")?.state).toBe("boundary")
    expect(getInlineAgents(events, false, t).find((agent) => agent.nodeId === "service")?.state).toBe("boundary")
    expect(getRuntimeNodeStates(events).get("data")?.state).toBe("boundary")
    expect(getRuntimeNodeStates(events).get("service")?.state).toBe("boundary")
  })
})

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}
