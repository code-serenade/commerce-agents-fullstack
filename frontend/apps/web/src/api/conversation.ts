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
  metadata: Record<string, unknown>
  createdAt: string
}

export interface PresignConversationFileUploadRequest {
  extension: string
  contentType?: string | null
}

export interface PresignConversationFileUploadResponse {
  method: "PUT"
  uploadUrl: string
  bucket: string
  objectKey: string
  contentType?: string | null
  expiresSecs?: number | null
}

export interface RegisterConversationFileRequest {
  messageId?: string | null
  objectKey: string
  fileName?: string | null
  mimeType?: string | null
  fileSize?: number | null
  sha256?: string | null
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

export interface UpdateConversationRequest {
  title?: string | null
}

export interface UpdateConversationFileExtractionRequest {
  status?: "pending" | "extracted" | "failed" | "unsupported" | null
  text?: string | null
  contentKind?: string | null
  source?: string | null
  parser?: string | null
  language?: string | null
  failureReason?: string | null
  metadata?: Record<string, unknown> | null
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

export const updateConversationApi = async (
  conversationId: string,
  body: UpdateConversationRequest
): Promise<ConversationResponse> => {
  return request<UpdateConversationRequest, ConversationResponse>({
    method: "PATCH",
    url: `/api/app/conversations/${conversationId}`,
    body,
  })
}

export const deleteConversationApi = async (
  conversationId: string
): Promise<void> => {
  return request<undefined, void>({
    method: "DELETE",
    url: `/api/app/conversations/${conversationId}`,
  })
}

export const presignConversationFileUploadApi = async (
  conversationId: string,
  body: PresignConversationFileUploadRequest
): Promise<PresignConversationFileUploadResponse> => {
  return request<
    PresignConversationFileUploadRequest,
    PresignConversationFileUploadResponse
  >({
    method: "POST",
    url: `/api/app/conversations/${conversationId}/files/presign-upload`,
    body,
  })
}

export const uploadConversationFileApi = async (
  file: File,
  sign: PresignConversationFileUploadResponse
): Promise<Response> => {
  const response = await fetch(sign.uploadUrl, {
    method: sign.method,
    headers: {
      "Content-Type": sign.contentType || file.type || "application/octet-stream",
    },
    body: file,
  })

  if (!response.ok) {
    throw new Error(`Upload failed: ${response.status}`)
  }

  return response
}

export const registerConversationFileApi = async (
  conversationId: string,
  body: RegisterConversationFileRequest
): Promise<ConversationFileResponse> => {
  return request<RegisterConversationFileRequest, ConversationFileResponse>({
    method: "POST",
    url: `/api/app/conversations/${conversationId}/files`,
    body,
  })
}

export const updateConversationFileExtractionApi = async (
  fileId: string,
  body: UpdateConversationFileExtractionRequest
): Promise<ConversationFileResponse> => {
  return request<
    UpdateConversationFileExtractionRequest,
    ConversationFileResponse
  >({
    method: "PUT",
    url: `/api/app/conversation-files/${fileId}/extraction`,
    body,
  })
}
