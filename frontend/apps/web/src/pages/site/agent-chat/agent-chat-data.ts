import {
  addConversationMessageApi,
  createConversationApi,
  deleteConversationApi,
  getAgentRuntimeOptionsApi,
  getConversationApi,
  getUploadDocumentSignApi,
  getUploadImageSignApi,
  listCommerceAgentPlaybooksApi,
  listCampaignsApi,
  listProductsApi,
  listConversationsApi,
  registerConversationFileApi,
  streamAgentSessionApi,
  submitAgentRunFeedbackApi,
  updateConversationApi,
  updateConversationFileExtractionApi,
  uploadWithSignedUrlApi,
} from "@/api"

export type { AgentRunFeedbackResponse, AgentRuntimeEvent } from "@/api"

export const AGENT_ID = "web-agent"

export const agentChatData = {
  addConversationMessage: addConversationMessageApi,
  createConversation: createConversationApi,
  deleteConversation: deleteConversationApi,
  getRuntimeOptions: getAgentRuntimeOptionsApi,
  getConversation: getConversationApi,
  listCampaigns: listCampaignsApi,
  listPlaybooks: listCommerceAgentPlaybooksApi,
  listProducts: listProductsApi,
  listConversations: listConversationsApi,
  getUploadDocumentSign: getUploadDocumentSignApi,
  getUploadImageSign: getUploadImageSignApi,
  registerConversationFile: registerConversationFileApi,
  streamAgentSession: streamAgentSessionApi,
  submitRunFeedback: submitAgentRunFeedbackApi,
  updateConversation: updateConversationApi,
  updateConversationFileExtraction: updateConversationFileExtractionApi,
  uploadWithSignedUrl: uploadWithSignedUrlApi,
}
