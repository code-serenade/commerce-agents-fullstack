export type AgentChatRole = "user" | "assistant"

export interface AgentChatMessage {
  id: string
  role: AgentChatRole
  content: string
  status?: "streaming" | "error"
}

export interface AgentChatConversationItem {
  id: string
  title: string
  updatedAt: string
}
