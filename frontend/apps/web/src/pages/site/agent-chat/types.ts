export type AgentChatRole = "user" | "assistant"

export interface AgentChatMessage {
  id: string
  role: AgentChatRole
  content: string
  status?: "streaming" | "error"
  activities?: AgentActivityItem[]
}

export interface AgentChatConversationItem {
  id: string
  title: string
  updatedAt: string
}

export type AgentActivityKind =
  | "session"
  | "memory"
  | "commander"
  | "worker"
  | "collaboration"
  | "task_graph"
  | "task_node"
  | "output"
  | "error"
  | "other"

export interface AgentActivityItem {
  id: string
  event: string
  kind: AgentActivityKind
  title: string
  description?: string
  workerId?: string
  taskId?: string
  nodeId?: string
  graphId?: string
  status?: "running" | "success" | "error" | "info"
  createdAt: string
  raw: unknown
}
