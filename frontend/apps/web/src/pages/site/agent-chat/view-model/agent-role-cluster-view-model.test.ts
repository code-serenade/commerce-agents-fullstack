import { describe, expect, it } from "vitest"
import {
  getAgentRoleActivities,
  getAgentRoleSummary,
} from "./agent-role-cluster-view-model"
import type { InlineAgent, InlineAgentState } from "./thread-inline-view-model"

const t = ((key: string, params?: Record<string, unknown>) => {
  if (key === "agentChat.thread.roleClusterActivity.summary.active") {
    return `active:${String(params?.active)}:${String(params?.total)}`
  }
  if (key === "agentChat.thread.roleClusterActivity.summary.done") {
    return `done:${String(params?.total)}`
  }
  if (key === "agentChat.thread.roleClusterActivity.summary.blocked") {
    return `blocked:${String(params?.total)}`
  }
  if (key === "agentChat.thread.roleClusterActivity.summary.waiting") {
    return `waiting:${String(params?.total)}`
  }
  return key
}) as never

describe("getAgentRoleActivities", () => {
  it("maps ecommerce roles to existing product role sprites and working actions", () => {
    const activities = getAgentRoleActivities(
      [
        createAgent("ops", "working"),
        createAgent("data", "working"),
        createAgent("creative", "working"),
        createAgent("service", "working"),
        createAgent("design", "working"),
        createAgent("engineering", "working"),
        createAgent("web", "working"),
        createAgent("ads-budget", "working"),
        createAgent("accounting", "working"),
        createAgent("delivery", "working"),
        createAgent("search", "working"),
      ],
      t
    )

    expect(activities.map((activity) => activity.role)).toEqual([
      "ops",
      "data",
      "creative",
      "service",
      "design",
      "engineering",
      "web",
      "accounting",
      "delivery",
      "search",
    ])
    expect(activities.map((activity) => activity.action)).toEqual([
      "agentChat.thread.roleClusterActivity.working.ops",
      "agentChat.thread.roleClusterActivity.working.data",
      "agentChat.thread.roleClusterActivity.working.creative",
      "agentChat.thread.roleClusterActivity.working.service",
      "agentChat.thread.roleClusterActivity.working.design",
      "agentChat.thread.roleClusterActivity.working.engineering",
      "agentChat.thread.roleClusterActivity.working.web",
      "agentChat.thread.roleClusterActivity.working.accounting",
      "agentChat.thread.roleClusterActivity.working.delivery",
      "agentChat.thread.roleClusterActivity.working.search",
    ])
    expect(activities[0]?.agent.name).toBe("agentChat.thread.agents.ops.name")
    expect(activities[0]?.contribution).toBe(
      "agentChat.thread.roleClusterActivity.contribution.ops"
    )
  })

  it("uses user-facing state actions without exposing runtime internals", () => {
    const activities = getAgentRoleActivities(
      [
        createAgent("data", "queued"),
        createAgent("creative", "reworking"),
        createAgent("service", "done"),
        createAgent("design", "boundary"),
        createAgent("engineering", "failed"),
        createAgent("web", "idle"),
      ],
      t
    )

    expect(activities.map((activity) => activity.action)).toEqual([
      "agentChat.thread.roleClusterActivity.actions.queued",
      "agentChat.thread.roleClusterActivity.actions.reworking",
      "agentChat.thread.roleClusterActivity.actions.done",
      "agentChat.thread.roleClusterActivity.actions.boundary",
      "agentChat.thread.roleClusterActivity.actions.failed",
      "agentChat.thread.roleClusterActivity.actions.idle",
    ])
  })

  it("maps platform launch wording to the existing ops role", () => {
    const activities = getAgentRoleActivities(
      [
        createAgent("readTarget", "working"),
        createAgent("douyin_ads", "working"),
        createAgent("qianchuan-plan", "working"),
        createAgent("ads-budget", "working"),
      ],
      t
    )

    expect(activities.map((activity) => activity.role)).toEqual(["box", "ops"])
    expect(activities[1]?.agent.name).toBe("agentChat.thread.agents.ops.name")
  })

  it("deduplicates several internal nodes that belong to the same user-facing role", () => {
    const activities = getAgentRoleActivities(
      [
        createAgent("douyin_ads", "done"),
        createAgent("qianchuan-plan", "working"),
        createAgent("ads-budget", "queued"),
      ],
      t
    )

    expect(activities).toHaveLength(1)
    expect(activities[0]?.role).toBe("ops")
    expect(activities[0]?.state).toBe("working")
    expect(activities[0]?.action).toBe(
      "agentChat.thread.roleClusterActivity.working.ops"
    )
  })

  it("keeps adjacent product roles visually distinct instead of folding them into creative", () => {
    const activities = getAgentRoleActivities(
      [
        createAgent("design-material-spec", "working"),
        createAgent("engineering-upload-ocr", "working"),
        createAgent("web-seo-keyword", "working"),
        createAgent("creative-video-script", "working"),
      ],
      t
    )

    expect(activities.map((activity) => activity.role)).toEqual([
      "design",
      "engineering",
      "web",
      "creative",
    ])
  })
})

describe("getAgentRoleSummary", () => {
  it("summarizes active, completed, blocked, and waiting groups", () => {
    expect(
      getAgentRoleSummary(
        getAgentRoleActivities(
          [createAgent("data", "working"), createAgent("ops", "queued")],
          t
        ),
        t
      )
    ).toBe("active:1:2")
    expect(
      getAgentRoleSummary(
        getAgentRoleActivities(
          [createAgent("data", "done"), createAgent("ops", "boundary")],
          t
        ),
        t
      )
    ).toBe("done:2")
    expect(
      getAgentRoleSummary(
        getAgentRoleActivities([createAgent("data", "failed")], t),
        t
      )
    ).toBe("blocked:1")
    expect(
      getAgentRoleSummary(
        getAgentRoleActivities([createAgent("data", "queued")], t),
        t
      )
    ).toBe("waiting:1")
  })
})

function createAgent(nodeId: string, state: InlineAgentState): InlineAgent {
  return {
    name: nodeId,
    nodeId,
    state,
    status: state,
    summary: `${nodeId} summary`,
  }
}
