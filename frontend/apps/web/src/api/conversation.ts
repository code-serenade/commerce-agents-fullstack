import { request } from "@workspace/services/api/base"
import type { WorkspaceArtifactResponse } from "./commerce"

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
  strategyId?: string | null
  responseMode?: string | null
  routingMode?: string | null
  collaborationMode?: string | null
  learningLevel?: string | null
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

export interface ConversationFileAccessResponse {
  file: ConversationFileResponse
  method: string
  downloadUrl: string
  expiresSecs: number
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
  strategyId?: string | null
  responseMode?: string | null
  routingMode?: string | null
  collaborationMode?: string | null
  learningLevel?: string | null
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

export interface TriggerConversationFileExtractionJobRequest {
  mode?: string | null
  force?: boolean
}

export interface ListUserFilesQuery {
  limit?: number | null
}

export interface AttachConversationFileRequest {
  conversationId: string
}

export interface BulkDeleteUserFilesRequest {
  fileIds: string[]
}

export type ConversationReportType =
  | "conversation_report"
  | "commerce_conversation_report"
  | "teaching_conversation_report"
  | "process_evidence_pack"

export type ConversationReportScope =
  | "conversation_all"
  | "conversation_recent"
  | "workspace_run"
  | "selected_sources"

export interface ExportConversationTranscriptRequest {
  format?: "markdown" | "text" | "json" | string
  includeFiles?: boolean
  includeRuntimeEvents?: boolean
  scope?: ConversationReportScope | string
}

export interface ConversationTranscriptExportResponse {
  conversationId: string
  fileName: string
  mimeType: string
  format: string
  content: string
  contentChars: number
  messageCount: number
  fileCount: number
  warnings: string[]
}

export interface GenerateConversationReportRequest {
  workspaceId?: string | null
  runId?: string | null
  reportType?: ConversationReportType | string
  scope?: ConversationReportScope | string
  format?: "markdown" | string
  includeProcess?: boolean
  includeSources?: boolean
  includeTranscriptAppendix?: boolean
  artifactIds?: string[]
  fileIds?: string[]
}

export interface GeneratedConversationReportExportResponse {
  fileName: string
  mimeType: string
  format: string
  content: string
  contentChars: number
  warnings: string[]
}

export interface GenerateConversationReportResponse {
  artifact: WorkspaceArtifactResponse
  export: GeneratedConversationReportExportResponse
}

export interface ConversationFileExtractionStatusResponse {
  fileId: string
  status: "pending" | "extracted" | "failed" | "unsupported" | string
  contentKind?: string | null
  parser?: string | null
  source?: string | null
  language?: string | null
  summary?: string | null
  textChars: number
  readable: boolean
  visionReady: boolean
  failureReason?: string | null
  metadata: Record<string, unknown>
  updatedAt: string
}

export interface ConversationFileExtractionJobResponse {
  jobId: string
  fileId: string
  status: "pending" | "extracted" | "failed" | "unsupported" | string
  contentKind?: string | null
  parser?: string | null
  source?: string | null
  message: string
  metadata: Record<string, unknown>
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

export const exportConversationTranscriptApi = async (
  conversationId: string,
  query: ExportConversationTranscriptRequest = {}
): Promise<ConversationTranscriptExportResponse> => {
  return request<
    ExportConversationTranscriptRequest,
    ConversationTranscriptExportResponse
  >({
    method: "GET",
    url: `/api/app/conversations/${conversationId}/transcript/export`,
    body: query,
  })
}

export const generateConversationReportApi = async (
  conversationId: string,
  body: GenerateConversationReportRequest
): Promise<GenerateConversationReportResponse> => {
  return request<
    GenerateConversationReportRequest,
    GenerateConversationReportResponse
  >({
    method: "POST",
    url: `/api/app/conversations/${conversationId}/reports/generate`,
    body,
  })
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

export const listUserFilesApi = async (
  query: ListUserFilesQuery = {}
): Promise<ConversationFileResponse[]> => {
  return request<ListUserFilesQuery, ConversationFileResponse[]>({
    method: "GET",
    url: "/api/app/files",
    body: query,
    suppressGlobalError: true,
  })
}

export const getConversationFileAccessApi = async (
  fileId: string
): Promise<ConversationFileAccessResponse> => {
  return request<undefined, ConversationFileAccessResponse>({
    method: "GET",
    url: `/api/app/conversation-files/${fileId}/access`,
    suppressGlobalError: true,
  })
}

export const attachConversationFileApi = async (
  fileId: string,
  body: AttachConversationFileRequest
): Promise<ConversationFileResponse> => {
  return request<AttachConversationFileRequest, ConversationFileResponse>({
    method: "POST",
    url: `/api/app/files/${fileId}/attach-to-conversation`,
    body,
  })
}

export const deleteUserFileApi = async (fileId: string): Promise<void> => {
  return request<undefined, void>({
    method: "DELETE",
    url: `/api/app/files/${fileId}`,
  })
}

export const bulkDeleteUserFilesApi = async (
  body: BulkDeleteUserFilesRequest
): Promise<void> => {
  return request<BulkDeleteUserFilesRequest, void>({
    method: "POST",
    url: "/api/app/files/bulk-delete",
    body,
  })
}

export const deleteConversationFileApi = async (
  fileId: string
): Promise<void> => {
  return request<undefined, void>({
    method: "DELETE",
    url: `/api/app/conversation-files/${fileId}`,
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

export const getConversationFileExtractionApi = async (
  fileId: string
): Promise<ConversationFileExtractionStatusResponse> => {
  return request<undefined, ConversationFileExtractionStatusResponse>({
    method: "GET",
    url: `/api/app/conversation-files/${fileId}/extraction`,
  })
}

export const triggerConversationFileExtractionJobApi = async (
  fileId: string,
  body: TriggerConversationFileExtractionJobRequest = {}
): Promise<ConversationFileExtractionJobResponse> => {
  return request<
    TriggerConversationFileExtractionJobRequest,
    ConversationFileExtractionJobResponse
  >({
    method: "POST",
    url: `/api/app/conversation-files/${fileId}/extraction-jobs`,
    body,
  })
}
