import { useMemo, useState } from "react"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import type { AgentChatMessage } from "../types"
import {
  findLatestProcessTarget,
  type AgentProcessTarget,
} from "./agent-process-visibility-view-model"

export function useAgentProcessPanel(input: {
  messages: AgentChatMessage[]
  runtimeEvents: AgentRuntimeEvent[]
  onOpen: (target: AgentProcessTarget) => void
}) {
  const { messages, onOpen, runtimeEvents } = input
  const [activeProcessTarget, setActiveProcessTarget] =
    useState<AgentProcessTarget | null>(null)
  const latestProcessTarget = useMemo(
    () => findLatestProcessTarget(messages, runtimeEvents),
    [messages, runtimeEvents]
  )
  const visibleProcessTarget = activeProcessTarget
  const activeProcessMessage = visibleProcessTarget
    ? messages.find((message) => message.id === visibleProcessTarget.messageId)
    : null
  const activeRuntimeEvents: AgentRuntimeEvent[] =
    activeProcessMessage?.runtimeEvents?.length
      ? activeProcessMessage.runtimeEvents
      : activeProcessMessage?.id === latestProcessTarget?.messageId
        ? runtimeEvents
        : []
  const shouldShowProcessPanel = Boolean(
    activeProcessMessage && activeRuntimeEvents.length > 0
  )

  const closeProcessPanel = () => {
    setActiveProcessTarget(null)
  }

  const openProcessPanel = (target: AgentProcessTarget) => {
    onOpen(target)
    setActiveProcessTarget(target)
  }

  return {
    activeProcessMessage,
    activeRuntimeEvents,
    closeProcessPanel,
    openProcessPanel,
    setActiveProcessTarget,
    shouldShowProcessPanel,
    visibleProcessTarget,
  }
}
