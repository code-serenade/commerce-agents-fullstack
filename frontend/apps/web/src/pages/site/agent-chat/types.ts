import type { AgentRuntimeEvent } from "./agent-chat-data"
import type { AgentChatStrategyId } from "./domain/agent-chat-strategy"

export type AgentChatRole = "user" | "assistant"

export interface AgentChatMessageContextReference {
  id: string
  label?: string
  secondary?: string
}

export interface AgentChatMessageContextSummary {
  campaignIds: string[]
  campaigns: AgentChatMessageContextReference[]
  productIds: string[]
  products: AgentChatMessageContextReference[]
  sourceRefs: string[]
  sources: AgentChatMessageContextReference[]
  strategyId?: AgentChatStrategyId
  collaborationMode?: string
  learningLevel?: string
  routingMode?: string
  primaryRole?: string
  selectedRoles: string[]
}

export interface AgentChatMessage {
  id: string
  role: AgentChatRole
  content: string
  contextSummary?: AgentChatMessageContextSummary
  runtimeEvents?: AgentRuntimeEvent[]
  status?: "streaming" | "error" | "stopped"
}

export interface AgentChatConversationItem {
  id: string
  title: string
  updatedAt: string
}

export interface AgentChatAttachmentItem {
  id: string
  fileId?: string
  imageDataUrl?: string
  imageDetail?: "auto" | "low" | "high"
  kind?: "image" | "file"
  name: string
  status: "uploading" | "uploaded" | "pending" | "failed" | "unsupported"
  readable: boolean
  unsupportedReason?: string
}

export interface AgentChatMessageStrategy {
  id: string
  responseMode: string
}
