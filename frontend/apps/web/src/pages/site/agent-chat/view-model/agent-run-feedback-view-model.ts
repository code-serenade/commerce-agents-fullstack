import type { AgentRuntimeEvent } from "../agent-chat-data"

export interface AgentRunFeedbackTarget {
  runId: string
  runStatus: string
  workspaceId: string
  workspaceTitle?: string
}

export function getAgentRunFeedbackTarget(
  runtimeEvents: AgentRuntimeEvent[]
): AgentRunFeedbackTarget | null {
  const completed = findLatest(runtimeEvents, "workspace_run_completed")
  if (completed) {
    const workspaceId = completed.workspaceId ?? completed.workspace_id
    if (workspaceId) {
      return {
        runId: completed.id,
        runStatus: completed.status,
        workspaceId,
      }
    }
  }

  const linked = findLatest(runtimeEvents, "workspace_run_linked")
  if (!linked) return null

  const workspaceId =
    linked.run.workspaceId ?? linked.run.workspace_id ?? linked.workspace.id
  if (!workspaceId) return null

  return {
    runId: linked.run.id,
    runStatus: linked.run.status,
    workspaceId,
    workspaceTitle: linked.workspace.title,
  }
}

function findLatest<TType extends AgentRuntimeEvent["type"]>(
  runtimeEvents: AgentRuntimeEvent[],
  type: TType
): Extract<AgentRuntimeEvent, { type: TType }> | null {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (event.type === type) {
      return event as Extract<AgentRuntimeEvent, { type: TType }>
    }
  }
  return null
}
