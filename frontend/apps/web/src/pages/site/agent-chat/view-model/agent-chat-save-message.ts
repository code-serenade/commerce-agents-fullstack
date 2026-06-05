import { agentChatData, type AgentRuntimeEvent } from "../agent-chat-data"

export function saveAssistantConversationMessage(input: {
  content: string
  conversationId: string
  runtimeEvents: AgentRuntimeEvent[]
}) {
  return agentChatData.addConversationMessage(input.conversationId, {
    role: "assistant",
    contentText: input.content,
    contentJson: input.runtimeEvents.length
      ? { runtimeEvents: input.runtimeEvents }
      : null,
  })
}
