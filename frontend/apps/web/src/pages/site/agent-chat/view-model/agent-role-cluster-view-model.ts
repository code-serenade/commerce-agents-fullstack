import type { TFunction } from "i18next"
import type { InlineAgent, InlineAgentState } from "./thread-inline-view-model"

export type AgentRoleKey =
  | "accounting"
  | "box"
  | "creative"
  | "data"
  | "delivery"
  | "design"
  | "engineering"
  | "ops"
  | "search"
  | "service"
  | "web"

export interface AgentRoleActivity {
  action: string
  agent: InlineAgent
  contribution: string
  lane: number
  role: AgentRoleKey
  state: InlineAgentState
}

export function getAgentRoleActivities(
  agents: InlineAgent[],
  t: TFunction
): AgentRoleActivity[] {
  const activitiesByRole = new Map<AgentRoleKey, AgentRoleActivity>()

  agents.forEach((agent, index) => {
    const role = getAgentRoleKey(agent.nodeId)
    const activity = {
      action: getAgentRoleAction(agent.state, role, t),
      agent: getDisplayAgent(agent, role, t),
      contribution: getAgentRoleContribution(role, t),
      lane: index % 3,
      role,
      state: agent.state,
    }

    const existing = activitiesByRole.get(role)
    activitiesByRole.set(
      role,
      existing ? mergeAgentRoleActivities(existing, activity, t) : activity
    )
  })

  return Array.from(activitiesByRole.values()).slice(0, 12).map((activity, index) => ({
    ...activity,
    lane: index % 3,
  })
  )
}

export function getAgentRoleSummary(
  activities: AgentRoleActivity[],
  t: TFunction
) {
  const total = activities.length
  const active = activities.filter((activity) =>
    activity.state === "working" || activity.state === "reworking"
  ).length

  if (activities.some((activity) => activity.state === "failed")) {
    return t("agentChat.thread.roleClusterActivity.summary.blocked", { total })
  }
  if (active > 0) {
    return t("agentChat.thread.roleClusterActivity.summary.active", { active, total })
  }
  if (
    total > 0 &&
    activities.every((activity) =>
      activity.state === "done" || activity.state === "boundary"
    )
  ) {
    return t("agentChat.thread.roleClusterActivity.summary.done", { total })
  }
  return t("agentChat.thread.roleClusterActivity.summary.waiting", { total })
}

function getAgentRoleKey(nodeId: string): AgentRoleKey {
  const normalized = nodeId.toLowerCase()
  const tokens = normalized.split(/[^a-z0-9]+/).filter(Boolean)
  if (
    normalized.includes("data") ||
    normalized.includes("metric") ||
    normalized.includes("analytics")
  ) {
    return "data"
  }
  if (
    normalized.includes("douyin") ||
    normalized.includes("qianchuan") ||
    hasToken(tokens, "ad", "ads", "advertising") ||
    normalized.includes("marketing")
  ) {
    return "ops"
  }
  if (
    normalized.includes("account") ||
    normalized.includes("finance") ||
    normalized.includes("budget")
  ) {
    return "accounting"
  }
  if (
    normalized.includes("engineering") ||
    normalized.includes("engineer") ||
    normalized.includes("technical") ||
    normalized.includes("architecture") ||
    normalized.includes("upload") ||
    normalized.includes("ocr")
  ) {
    return "engineering"
  }
  if (
    normalized.includes("design") ||
    normalized.includes("visual") ||
    normalized.includes("layout") ||
    normalized.includes("ux") ||
    normalized.includes("ui")
  ) {
    return "design"
  }
  if (
    hasToken(tokens, "web", "seo") ||
    normalized.includes("keyword") ||
    normalized.includes("traffic")
  ) {
    return "web"
  }
  if (
    normalized.includes("creative") ||
    normalized.includes("copy") ||
    normalized.includes("script") ||
    normalized.includes("content")
  ) {
    return "creative"
  }
  if (
    normalized.includes("service") ||
    normalized.includes("customer") ||
    normalized.includes("support")
  ) {
    return "service"
  }
  if (
    normalized.includes("search") ||
    normalized.includes("rag") ||
    normalized.includes("source") ||
    normalized.includes("evidence") ||
    normalized.includes("material") ||
    normalized.includes("policy") ||
    normalized.includes("rule") ||
    normalized.includes("compliance") ||
    normalized.includes("boundary") ||
    normalized.includes("target") ||
    normalized.includes("verify") ||
    normalized.includes("check") ||
    normalized.includes("research") ||
    normalized.includes("read")
  ) {
    return "search"
  }
  if (
    normalized.includes("delivery") ||
    normalized.includes("report") ||
    normalized.includes("document") ||
    normalized.includes("handoff") ||
    normalized.includes("output")
  ) {
    return "delivery"
  }
  if (
    normalized.includes("product") ||
    normalized.includes("sku") ||
    normalized.includes("item") ||
    normalized.includes("listing") ||
    normalized.includes("title") ||
    normalized.includes("price") ||
    normalized.includes("category") ||
    normalized.includes("selection")
  ) {
    return "box"
  }
  if (
    normalized.includes("ops") ||
    normalized.includes("operation") ||
    normalized.includes("campaign")
  ) {
    return "ops"
  }
  return "search"
}

function getDisplayAgent(
  agent: InlineAgent,
  role: AgentRoleKey,
  t: TFunction
): InlineAgent {
  return {
    ...agent,
    name: t(`agentChat.thread.agents.${role}.name`),
    summary: t(`agentChat.thread.agents.${role}.summary`),
  }
}

function mergeAgentRoleActivities(
  current: AgentRoleActivity,
  next: AgentRoleActivity,
  t: TFunction
): AgentRoleActivity {
  const state =
    getAgentStatePriority(next.state) > getAgentStatePriority(current.state)
      ? next.state
      : current.state

  return {
    ...current,
    action: getAgentRoleAction(state, current.role, t),
    agent: {
      ...current.agent,
      state,
      status: t(`agentChat.thread.inlineState.${state}`),
    },
    state,
  }
}

function getAgentStatePriority(state: InlineAgentState) {
  if (state === "failed") return 70
  if (state === "working" || state === "reworking") return 60
  if (state === "boundary") return 50
  if (state === "queued") return 40
  if (state === "done") return 30
  return 10
}

function hasToken(tokens: string[], ...candidates: string[]) {
  return candidates.some((candidate) => tokens.includes(candidate))
}

function getAgentRoleContribution(role: AgentRoleKey, t: TFunction) {
  return t(`agentChat.thread.roleClusterActivity.contribution.${role}`)
}

function getAgentRoleAction(
  state: InlineAgentState,
  role: AgentRoleKey,
  t: TFunction
) {
  if (state === "working") {
    return t(`agentChat.thread.roleClusterActivity.working.${role}`)
  }
  if (state === "reworking") {
    return t("agentChat.thread.roleClusterActivity.actions.reworking")
  }
  if (state === "done") {
    return t("agentChat.thread.roleClusterActivity.actions.done")
  }
  if (state === "boundary") {
    return t("agentChat.thread.roleClusterActivity.actions.boundary")
  }
  if (state === "failed") {
    return t("agentChat.thread.roleClusterActivity.actions.failed")
  }
  if (state === "queued") {
    return t("agentChat.thread.roleClusterActivity.actions.queued")
  }
  return t("agentChat.thread.roleClusterActivity.actions.idle")
}
