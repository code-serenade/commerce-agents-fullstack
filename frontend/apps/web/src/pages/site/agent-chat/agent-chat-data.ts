import {
  addConversationMessageApi,
  createConversationApi,
  deleteConversationApi,
  getConversationApi,
  listProductsApi,
  listConversationsApi,
  streamAgentSessionApi,
  updateConversationApi,
} from "@/api"

export type { AgentRuntimeEvent } from "@/api"

export const AGENT_ID = "web-agent"
export const WORKSPACE_ID = "workspace-001"

export const agentChatData = {
  addConversationMessage: addConversationMessageApi,
  createConversation: createConversationApi,
  deleteConversation: deleteConversationApi,
  getConversation: getConversationApi,
  listProducts: listProductsApi,
  listConversations: listConversationsApi,
  streamAgentSession: streamAgentSessionApi,
  updateConversation: updateConversationApi,
}
