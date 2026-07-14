import type { AgentChatMessage } from "../types"

export interface VisibleThreadMessageEntry {
  index: number
  message: AgentChatMessage
}

export function getVisibleThreadMessageEntries(
  messages: AgentChatMessage[]
): VisibleThreadMessageEntry[] {
  return messages
    .map((message, index) => ({ index, message }))
    .filter(({ message }) => message.id !== "welcome")
}

export function getThreadMessageIdentityKey(messages: AgentChatMessage[]) {
  return messages.map((message) => message.id).join("|")
}
