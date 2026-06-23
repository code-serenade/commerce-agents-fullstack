import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getInlineAgents } from "./thread-inline-view-model"

const internal = {
  provider: ["external", "search", "provider"].join(" "),
  routeMode: ["route", "mode"].join("_"),
  selectedSkills: ["selected", "skills"].join("_"),
  skillId: ["skill", "id"].join("_"),
  taskGraph: ["task", "graph"].join("_"),
  url: ["http:", "", "internal", "search"].join("/"),
  worker: "worker",
}

const t = ((key: string) => {
  if (key === "agentChat.thread.agents.data.name") return "data"
  if (key === "agentChat.thread.agents.data.summary") return "data summary"
  if (key === "agentChat.thread.agents.delivery.name") return "delivery"
  if (key === "agentChat.thread.agents.delivery.summary") return "delivery summary"
  if (key === "agentChat.thread.steps.userFacingTitles.generic") {
    return "organize judgment"
  }
  if (key === "agentChat.thread.steps.userFacingDetail") {
    return "user facing detail"
  }
  if (key === "agentChat.thread.inlineState.queued") return "queued"
  if (key === "agentChat.thread.inlineState.idle") return "idle"
  return key
}) as never

describe("getInlineAgents user-facing text", () => {
  it("does not expose backend node title or objective internals", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: "sunscreen title analysis",
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: `${internal.worker}.data`,
              title: `${internal.worker}.data ${internal.routeMode} ${internal.taskGraph}`,
              objective: `${internal.skillId} query_product_metrics should check raw strategy`,
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
            {
              id: "unknown_step",
              role: "unknown",
              worker_id: `${internal.worker}.unknown`,
              title: `${internal.worker}.unknown ${internal.selectedSkills}`,
              objective: `${internal.provider} ${internal.url}`,
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
          ],
        },
      },
    ]

    const agents = getInlineAgents(events, false, t)
    const visibleText = agents
      .map((agent) => `${agent.name} ${agent.summary}`)
      .join("\n")

    expect(agents.find((agent) => agent.nodeId === "data")?.name).toBe("data")
    expect(visibleText).toContain("data summary")
    expect(visibleText).toContain("organize judgment")
    expect(visibleText).toContain("user facing detail")
    expect(visibleText).not.toContain(internal.worker)
    expect(visibleText).not.toContain(internal.skillId)
    expect(visibleText).not.toContain(internal.routeMode)
    expect(visibleText).not.toContain(internal.taskGraph)
    expect(visibleText).not.toContain(internal.selectedSkills)
    expect(visibleText).not.toContain(internal.provider)
    expect(visibleText).not.toContain(internal.url)
  })
})
