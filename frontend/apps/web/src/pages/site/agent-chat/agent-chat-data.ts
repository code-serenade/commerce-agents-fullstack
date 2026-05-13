import {
  addConversationMessageApi,
  createConversationApi,
  deleteConversationApi,
  getAgentRuntimeOptionsApi,
  getConversationApi,
  listCommerceAgentPlaybooksApi,
  listProductsApi,
  listConversationsApi,
  presignConversationFileUploadApi,
  registerConversationFileApi,
  streamAgentSessionApi,
  submitAgentRunFeedbackApi,
  updateConversationApi,
  updateConversationFileExtractionApi,
  uploadConversationFileApi,
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
  presignConversationFileUpload: presignConversationFileUploadApi,
  registerConversationFile: registerConversationFileApi,
  streamAgentSession: streamAgentSessionApi,
  submitRunFeedback: submitAgentRunFeedbackApi,
  updateConversation: updateConversationApi,
  updateConversationFileExtraction: updateConversationFileExtractionApi,
  uploadConversationFile: uploadConversationFileApi,
}
