import {
  addConversationMessageApi,
  createConversationApi,
  deleteConversationApi,
  getConversationApi,
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
  listConversations: listConversationsApi,
  streamAgentSession: streamAgentSessionApi,
  updateConversation: updateConversationApi,
}
