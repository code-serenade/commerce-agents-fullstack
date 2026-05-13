import type { AgentRuntimeEvent } from "./agent-chat-data"

export type AgentChatRole = "user" | "assistant"

export interface AgentChatMessage {
  id: string
  role: AgentChatRole
  content: string
  runtimeEvents?: AgentRuntimeEvent[]
  status?: "streaming" | "error"
}

export interface AgentChatConversationItem {
  id: string
  title: string
  updatedAt: string
}

export interface AgentChatAttachmentItem {
  id: string
  fileId?: string
  name: string
  status: "uploading" | "uploaded" | "failed"
  readable: boolean
}

export interface AgentChatMessageStrategy {
  id: string
  responseMode: string
  collaborationMode: string
}
