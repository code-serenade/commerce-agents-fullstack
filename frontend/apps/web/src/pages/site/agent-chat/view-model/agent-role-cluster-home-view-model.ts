import type { TFunction } from "i18next"
import type { InlineAgent, InlineAgentState } from "./thread-inline-view-model"

const HOME_AGENT_PREVIEW = [
  ["ops", "ops", "working"],
  ["search", "search", "working"],
  ["data", "data", "working"],
  ["creative", "creative", "working"],
  ["design", "design", "working"],
] as const satisfies readonly (readonly [string, string, InlineAgentState])[]

export function getHomeRoleClusterAgents(t: TFunction): InlineAgent[] {
  return HOME_AGENT_PREVIEW.map(([nodeId, key, state]) => ({
    name: t(`agentChat.thread.agents.${key}.name`),
    nodeId,
    state,
    status: t(`agentChat.thread.inlineState.${state}`),
    summary: t(`agentChat.thread.agents.${key}.summary`),
  }))
}
