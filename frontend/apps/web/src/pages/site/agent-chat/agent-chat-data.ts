import {
  addConversationMessageApi,
  createConversationApi,
  deleteConversationApi,
  getAgentRuntimeOptionsApi,
  getConversationFileExtractionApi,
  getConversationApi,
  listCommerceAgentPlaybooksApi,
  listCampaignsApi,
  listProductsApi,
  listConversationsApi,
  presignConversationFileUploadApi,
  registerConversationFileApi,
  streamAgentSessionApi,
  submitAgentRunFeedbackApi,
  triggerConversationFileExtractionJobApi,
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
  getConversationFileExtraction: getConversationFileExtractionApi,
  getConversation: getConversationApi,
  listCampaigns: listCampaignsApi,
  listPlaybooks: listCommerceAgentPlaybooksApi,
  listProducts: listProductsApi,
  listConversations: listConversationsApi,
  presignConversationFileUpload: presignConversationFileUploadApi,
  registerConversationFile: registerConversationFileApi,
  streamAgentSession: streamAgentSessionApi,
  submitRunFeedback: submitAgentRunFeedbackApi,
  triggerConversationFileExtractionJob: triggerConversationFileExtractionJobApi,
  updateConversation: updateConversationApi,
  updateConversationFileExtraction: updateConversationFileExtractionApi,
  uploadConversationFile: uploadConversationFileApi,
}
