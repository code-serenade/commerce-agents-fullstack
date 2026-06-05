import type { Dispatch, SetStateAction } from "react"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import type { AgentChatMessage } from "../types"

export function markAssistantAnswerComplete(input: {
  assistantId: string
  content: string
  runtimeEvents: AgentRuntimeEvent[]
  setMessages: Dispatch<SetStateAction<AgentChatMessage[]>>
}) {
  input.setMessages((current) =>
    current.map((item) =>
      item.id === input.assistantId
        ? {
            ...item,
            content: item.content || input.content,
            runtimeEvents: [...input.runtimeEvents],
            status: undefined,
          }
        : item
    )
  )
}

export function replaceAssistantAnswerComplete(input: {
  assistantId: string
  content: string
  runtimeEvents: AgentRuntimeEvent[]
  setMessages: Dispatch<SetStateAction<AgentChatMessage[]>>
}) {
  input.setMessages((current) =>
    current.map((message) =>
      message.id === input.assistantId
        ? {
            ...message,
            content: input.content,
            runtimeEvents: [...input.runtimeEvents],
            status: undefined,
          }
        : message
    )
  )
}
