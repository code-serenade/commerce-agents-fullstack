import { describe, expect, it } from "vitest"
import { getHomeRoleClusterAgents } from "./agent-role-cluster-home-view-model"

const t = ((key: string) => key) as never

describe("getHomeRoleClusterAgents", () => {
  it("keeps the first-screen preview focused on user-facing commerce roles", () => {
    const agents = getHomeRoleClusterAgents(t)

    expect(agents.map((agent) => agent.nodeId)).toEqual([
      "ops",
      "search",
      "data",
      "creative",
      "design",
    ])
    expect(agents.map((agent) => agent.state)).toEqual([
      "working",
      "working",
      "working",
      "working",
      "working",
    ])
    expect(agents.map((agent) => agent.name)).toEqual([
      "agentChat.thread.agents.ops.name",
      "agentChat.thread.agents.search.name",
      "agentChat.thread.agents.data.name",
      "agentChat.thread.agents.creative.name",
      "agentChat.thread.agents.design.name",
    ])
  })
})
