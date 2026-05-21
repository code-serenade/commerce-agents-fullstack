import type { ReactNode } from "react"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentChatMessage } from "../types"
import {
  findPreviousUserTask,
  findTurnLabel,
  getProcessRuntimeEvents,
  shouldRenderMessageProcess,
  type AgentProcessTarget,
} from "../view-model/agent-process-visibility-view-model"
import { InlineSwarmTaskCard } from "./inline-swarm-task-card"

export function AgentChatThreadTurn({
  activeProcessMessageId,
  index,
  inlineIsStreaming,
  message,
  messages,
  onCloseProcessPanel,
  onOpenProcessPanel,
  renderMessage,
  runtimeEvents,
  showProcessPanelAction,
}: {
  activeProcessMessageId: string | null
  index: number
  inlineIsStreaming: boolean
  message: AgentChatMessage
  messages: AgentChatMessage[]
  onCloseProcessPanel: () => void
  onOpenProcessPanel: (target: AgentProcessTarget) => void
  renderMessage: (message: AgentChatMessage) => ReactNode
  runtimeEvents: AgentRuntimeEvent[]
  showProcessPanelAction: boolean
}) {
  const processEvents = getProcessRuntimeEvents(messages, index, runtimeEvents)
  const task = findPreviousUserTask(messages, index)

  return (
    <div data-message-id={message.id} className="min-w-0 space-y-3">
      {shouldRenderMessageProcess(message, processEvents) ? (
        <InlineSwarmTaskCard
          messageId={message.id}
          isStreaming={message.status === "streaming" && inlineIsStreaming}
          onCloseProcessPanel={onCloseProcessPanel}
          onOpenProcessPanel={() =>
            onOpenProcessPanel({
              anchorLabel: findTurnLabel(messages, index),
              messageId: message.id,
              task,
            })
          }
          processPanelOpen={activeProcessMessageId === message.id}
          runtimeEvents={processEvents}
          showProcessPanelAction={showProcessPanelAction}
          task={task}
        />
      ) : null}
      {renderMessage(message)}
    </div>
  )
}
