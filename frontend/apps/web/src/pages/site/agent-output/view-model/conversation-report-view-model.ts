import type { TFunction } from "i18next"
import type {
  ConversationReportScope,
  ConversationReportType,
  GenerateConversationReportRequest,
  JsonRecord,
  WorkspaceArtifactResponse,
} from "@/api"
import type { AgentOutputDeliverable } from "../agent-output-backend-data"

export interface ConversationReportContext {
  conversationId: string | null
  runId: string | null
  workspaceId: string | null
}

export interface ConversationReportDraft {
  includeProcess: boolean
  includeSources: boolean
  includeTranscriptAppendix: boolean
  reportType: ConversationReportType
  scope: ConversationReportScope
}

export const defaultConversationReportDraft: ConversationReportDraft = {
  includeProcess: true,
  includeSources: true,
  includeTranscriptAppendix: false,
  reportType: "commerce_conversation_report",
  scope: "conversation_all",
}

export function createConversationReportRequest(
  draft: ConversationReportDraft,
  context: ConversationReportContext
): GenerateConversationReportRequest {
  return {
    format: "markdown",
    includeProcess: draft.includeProcess,
    includeSources: draft.includeSources,
    includeTranscriptAppendix: draft.includeTranscriptAppendix,
    reportType: draft.reportType,
    runId: context.runId,
    scope: draft.scope,
    workspaceId: context.workspaceId,
  }
}

export function getConversationReportContext(
  params: URLSearchParams,
  selectedArtifact: AgentOutputDeliverable | null | undefined
): ConversationReportContext {
  return {
    conversationId:
      readSearchParam(params, "conversationId") ??
      readMetadataString(selectedArtifact?.metadata, "conversationId"),
    runId:
      readSearchParam(params, "runId") ??
      readArtifactString(selectedArtifact, "runId") ??
      readMetadataString(selectedArtifact?.metadata, "runId"),
    workspaceId:
      readSearchParam(params, "workspaceId") ??
      readArtifactString(selectedArtifact, "workspaceId") ??
      readMetadataString(selectedArtifact?.metadata, "workspaceId"),
  }
}

export function isConversationReportArtifact(
  artifact: WorkspaceArtifactResponse | null | undefined,
  conversationId: string | null | undefined
) {
  const id = conversationId?.trim()
  if (!id) return false

  return readMetadataString(artifact?.metadata, "conversationId") === id
}

export function getConversationReportTypeOptions(t: TFunction) {
  const types: ConversationReportType[] = [
    "commerce_conversation_report",
    "teaching_conversation_report",
    "conversation_report",
    "process_evidence_pack",
  ]

  return types.map((type) => ({
    label: t(`agentOutput.backend.reportDialog.reportTypes.${type}`),
    value: type,
  }))
}

export function getConversationReportScopeOptions(t: TFunction) {
  const scopes: ConversationReportScope[] = [
    "conversation_all",
    "conversation_recent",
    "workspace_run",
    "selected_sources",
  ]

  return scopes.map((scope) => ({
    label: t(`agentOutput.backend.reportDialog.scopes.${scope}`),
    value: scope,
  }))
}

function readSearchParam(params: URLSearchParams, key: string) {
  const value = params.get(key)?.trim()
  return value || null
}

function readArtifactString(
  artifact: WorkspaceArtifactResponse | null | undefined,
  key: keyof Pick<WorkspaceArtifactResponse, "runId" | "workspaceId">
) {
  const value = artifact?.[key]
  return typeof value === "string" && value.trim() ? value.trim() : null
}

function readMetadataString(
  metadata: JsonRecord | null | undefined,
  key: string
) {
  const value = metadata?.[key]
  return typeof value === "string" && value.trim() ? value.trim() : null
}
