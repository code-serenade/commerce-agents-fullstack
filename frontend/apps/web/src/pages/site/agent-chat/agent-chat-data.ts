import {
  addConversationMessageApi,
  createConversationApi,
  deleteConversationApi,
  getAgentRuntimeOptionsApi,
  getConversationApi,
  listCommerceAgentPlaybooksApi,
  listProductsApi,
  listConversationsApi,
  streamAgentSessionApi,
  submitAgentRunFeedbackApi,
  updateConversationApi,
} from "@/api"

export type { AgentRunFeedbackResponse, AgentRuntimeEvent } from "@/api"

export const AGENT_ID = "web-agent"

export const agentChatData = {
  addConversationMessage: addConversationMessageApi,
  createConversation: createConversationApi,
  deleteConversation: deleteConversationApi,
  getRuntimeOptions: getAgentRuntimeOptionsApi,
  getConversation: getConversationApi,
  listPlaybooks: listCommerceAgentPlaybooksApi,
  listProducts: listProductsApi,
  listConversations: listConversationsApi,
  streamAgentSession: streamAgentSessionApi,
  submitRunFeedback: submitAgentRunFeedbackApi,
  updateConversation: updateConversationApi,
}
