import type { AgentRuntimeEvent } from "@/api"
import type { AgentChatMessage } from "../types"

export interface AgentProcessTarget {
  anchorLabel: string
  messageId: string
  task: string
}

export function getProcessRuntimeEvents(
  messages: AgentChatMessage[],
  messageIndex: number,
  liveRuntimeEvents: AgentRuntimeEvent[]
) {
  const messageEvents = messages[messageIndex]?.runtimeEvents ?? []
  if (messageEvents.length > 0) return messageEvents
  if (!isLatestAssistantTurn(messages, messageIndex)) return []
  return liveRuntimeEvents
}

export function shouldRenderMessageProcess(
  message: AgentChatMessage,
  runtimeEvents: AgentRuntimeEvent[]
) {
  return (
    message.role === "assistant" &&
    message.id !== "welcome" &&
    (runtimeEvents.length > 0 || message.status === "streaming")
  )
}

export function findLatestProcessTarget(
  messages: AgentChatMessage[],
  liveRuntimeEvents: AgentRuntimeEvent[]
): AgentProcessTarget | null {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index]
    if (!message || message.id === "welcome") continue
    const runtimeEvents = getProcessRuntimeEvents(messages, index, liveRuntimeEvents)
    if (!shouldRenderMessageProcess(message, runtimeEvents)) continue
    return {
      anchorLabel: findTurnLabel(messages, index),
      messageId: message.id,
      task: findPreviousUserTask(messages, index),
    }
  }
  return null
}

export function findPreviousUserTask(
  messages: AgentChatMessage[],
  fromIndex: number
) {
  return findPreviousUserContent(messages, fromIndex) ?? ""
}

export function findPreviousUserContent(
  messages: AgentChatMessage[],
  fromIndex: number
) {
  for (let index = fromIndex - 1; index >= 0; index -= 1) {
    const message = messages[index]
    if (message?.role === "user" && message.content.trim()) {
      return message.content
    }
  }
  return undefined
}

export function findTurnLabel(messages: AgentChatMessage[], fromIndex: number) {
  const turn = messages
    .slice(0, fromIndex + 1)
    .filter((message) => message.role === "assistant" && message.id !== "welcome")
    .length
  return `#${String(Math.max(turn, 1)).padStart(2, "0")}`
}

function isLatestAssistantTurn(
  messages: AgentChatMessage[],
  messageIndex: number
) {
  const message = messages[messageIndex]
  if (message?.role !== "assistant" || message.id === "welcome") return false
  for (let index = messageIndex + 1; index < messages.length; index += 1) {
    if (messages[index]?.role === "assistant" && messages[index]?.id !== "welcome") {
      return false
    }
  }
  return true
}
