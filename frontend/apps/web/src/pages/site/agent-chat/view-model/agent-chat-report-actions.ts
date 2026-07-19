import type { AgentRuntimeEvent } from "../agent-chat-data"
import { getAgentRunFeedbackTarget } from "./agent-run-feedback-view-model"

export interface AgentChatReportRouteContext {
  conversationId: string
  runId?: string
  workspaceId?: string
}

export function getAgentChatReportRouteContext(input: {
  conversationId: string | null
  runtimeEvents: AgentRuntimeEvent[]
}): AgentChatReportRouteContext | null {
  if (!input.conversationId) return null
  const target = getAgentRunFeedbackTarget(input.runtimeEvents)

  return {
    conversationId: input.conversationId,
    runId: target?.runId,
    workspaceId: target?.workspaceId,
  }
}

export function createAgentOutputReportUrl(
  context: AgentChatReportRouteContext
) {
  const params = new URLSearchParams({
    conversationId: context.conversationId,
  })
  if (context.workspaceId) params.set("workspaceId", context.workspaceId)
  if (context.runId) params.set("runId", context.runId)

  return `/agent-output?${params.toString()}`
}
