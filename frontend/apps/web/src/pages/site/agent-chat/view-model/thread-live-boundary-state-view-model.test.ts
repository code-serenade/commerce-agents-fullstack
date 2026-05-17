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
  if (key === "agentChat.thread.agents.data.name") return "data"
  if (key === "agentChat.thread.agents.data.summary") return "data summary"
  if (key === "agentChat.thread.agents.service.name") return "service"
  if (key === "agentChat.thread.agents.service.summary") return "service summary"
  return key
}) as never

const liveConversionNoPresetTask = [
  textFromCodes([35831, 29992, 19968, 21477, 35805, 35828, 26126]),
  textFromCodes([22914, 20309, 21028, 26029]),
  textFromCodes([30452, 25773, 38388, 36716, 21270]),
  textFromCodes([26159, 21542, 20581, 24247]),
  textFromCodes([65292]),
  textFromCodes([19981, 24341, 29992, 20219, 20309, 39044, 35774, 21830, 21697]),
].join("")

describe("live conversion no-product boundary display", () => {
  it("downgrades no-preset-product quality gaps to boundary states", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-live",
          root_task: liveConversionNoPresetTask,
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: "worker.data",
              title: ["Data", "diagnosis"].join(" "),
              objective: "live conversion data boundary",
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
            {
              id: "service",
              role: "service",
              worker_id: "worker.service",
              title: ["Service", "support"].join(" "),
              objective: "live conversion service boundary",
              input_refs: ["data"],
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
      { type: "task_graph_finished", graph_id: "graph-live" },
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
