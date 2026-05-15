import type { AgentRuntimeEvent } from "./agent-chat-data"
import type { AgentChatStrategyId } from "./domain/agent-chat-strategy"

export type AgentChatRole = "user" | "assistant"

export interface AgentChatMessageContextSummary {
  productIds: string[]
  sourceRefs: string[]
  strategyId?: AgentChatStrategyId
  collaborationMode?: string
  learningLevel?: string
  modelName?: string
  primaryRole?: string
  selectedRoles: string[]
}

export interface AgentChatMessage {
  id: string
  role: AgentChatRole
  content: string
  contextSummary?: AgentChatMessageContextSummary
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
