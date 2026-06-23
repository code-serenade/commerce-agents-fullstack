import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getInlineProcessStage } from "./thread-inline-view-model"

const t = ((key: string) => {
  if (key === "agentChat.thread.processStage.finished") return "finished"
  if (key === "agentChat.thread.processStage.failed") return "failed"
  return key
}) as never

describe("getInlineProcessStage failure priority", () => {
  it("shows blocking timeout as failed even after task graph finished", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: "analyze selected product and materials",
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: "worker.data",
              title: ["Data", "diagnosis"].join(" "),
              objective: "selected product data diagnosis",
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
        reason: "worker cognition timed out",
      },
      { type: "task_graph_finished", graph_id: "graph-1" },
    ]

    expect(
      getInlineProcessStage({
        hasRuntime: true,
        isStreaming: false,
        runtimeEvents: events,
      }, t)
    ).toBe("failed")
  })
})
