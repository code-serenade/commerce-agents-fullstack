import { request } from "@workspace/services/api/base"

export interface ConversationResponse {
  id: string
  userId: string
  title: string | null
  status: string
  source: string
  agentId: string | null
  modelName: string | null
  createdAt: string
  updatedAt: string
}

export interface ConversationMessageResponse {
  id: string
  conversationId: string
  userId: string
  role: "user" | "assistant" | "system" | "tool"
  contentText: string | null
  contentJson: unknown | null
  status: string
  sequence: number
  createdAt: string
  updatedAt: string
}

export interface ConversationFileResponse {
  id: string
  conversationId: string
  messageId: string | null
  bucket: string
  objectKey: string
  fileName: string | null
  mimeType: string | null
  fileSize: number | null
  sha256: string | null
  status: string
  createdAt: string
}

export interface ConversationDetailResponse {
  conversation: ConversationResponse
  messages: ConversationMessageResponse[]
  files: ConversationFileResponse[]
}

export interface CreateConversationRequest {
  title?: string | null
  firstMessage?: string | null
  agentId?: string | null
}

export interface AddConversationMessageRequest {
  role: "user" | "assistant" | "system" | "tool"
  contentText?: string | null
  contentJson?: unknown | null
}

export const listConversationsApi = async (): Promise<ConversationResponse[]> => {
  return request<undefined, ConversationResponse[]>({
    method: "GET",
    url: "/api/app/conversations",
    suppressGlobalError: true,
  })
}

export const createConversationApi = async (
  body: CreateConversationRequest
): Promise<ConversationDetailResponse> => {
  return request<CreateConversationRequest, ConversationDetailResponse>({
    method: "POST",
    url: "/api/app/conversations",
    body,
  })
}

export const getConversationApi = async (
  conversationId: string
): Promise<ConversationDetailResponse> => {
  return request<undefined, ConversationDetailResponse>({
    method: "GET",
    url: `/api/app/conversations/${conversationId}`,
    suppressGlobalError: true,
  })
}

export const addConversationMessageApi = async (
  conversationId: string,
  body: AddConversationMessageRequest
): Promise<ConversationMessageResponse> => {
  return request<AddConversationMessageRequest, ConversationMessageResponse>({
    method: "POST",
    url: `/api/app/conversations/${conversationId}/messages`,
    body,
  })
}
