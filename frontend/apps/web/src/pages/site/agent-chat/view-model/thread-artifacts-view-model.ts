import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { InlineAgentState } from "./thread-inline-view-model"
import { isInternalRuntimeText } from "./agent-user-facing-runtime-text"

export interface InlineArtifact {
  filename: string
  id: string
  meta: string
  nodeId: string
  status: string
  state: InlineAgentState
}

export function getInlineArtifactsFromRuntime(
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
): InlineArtifact[] {
  return runtimeEvents
    .filter(
      (
        event
      ): event is Extract<
        AgentRuntimeEvent,
        { type: "workspace_artifact_created" }
      > => event.type === "workspace_artifact_created"
    )
    .map((event) => ({
      filename: readArtifactFilename(event, t),
      id: event.id,
      meta: readArtifactMeta(event, t),
      nodeId: "delivery",
      status: readArtifactStatus(event, t),
      state: readArtifactState(event.status),
    }))
}

function readArtifactFilename(
  event: Extract<AgentRuntimeEvent, { type: "workspace_artifact_created" }>,
  t: TFunction
) {
  return sanitizeArtifactText(
    event.fileName ?? event.file_name ?? event.title,
    t("agentChat.thread.artifacts.fileFallback")
  )
}

function readArtifactMeta(
  event: Extract<AgentRuntimeEvent, { type: "workspace_artifact_created" }>,
  t: TFunction
) {
  return formatArtifactType(
    event.artifactType ??
      event.artifact_type ??
      event.format ??
      event.storageKind ??
      event.storage_kind ??
      "artifact",
    t
  )
}

function readArtifactStatus(
  event: Extract<AgentRuntimeEvent, { type: "workspace_artifact_created" }>,
  t: TFunction
) {
  const artifactType = event.artifactType ?? event.artifact_type ?? "artifact"
  const preview = event.preview?.trim()
  if (isSearchEvidenceArtifact(artifactType)) {
    return t("agentChat.thread.artifacts.searchEvidence")
  }
  const readablePreview = sanitizeArtifactText(preview, "")
  if (readablePreview) return readablePreview
  return t("agentChat.thread.artifacts.backendCreated", {
    status: formatArtifactStatus(event.status, t),
    type: formatArtifactType(artifactType, t),
  })
}

function formatArtifactType(type: string | undefined, t: TFunction) {
  const normalized = String(type ?? "").trim().toLowerCase()
  if (isSearchEvidenceArtifact(normalized)) {
    return t("agentChat.thread.artifacts.searchEvidenceMeta")
  }
  if (normalized === "final_report") {
    return t("agentChat.thread.artifacts.finalReportMeta")
  }
  if (normalized === "artifact") {
    return t("agentChat.thread.artifacts.genericMeta")
  }
  if (!normalized || isInternalRuntimeText(normalized) || isMachineArtifactType(normalized)) {
    return t("agentChat.thread.artifacts.genericMeta")
  }
  return type?.trim() ?? t("agentChat.thread.artifacts.genericMeta")
}

function formatArtifactStatus(status: string, t: TFunction) {
  if (status === "active" || status === "completed" || status === "success") {
    return t("agentChat.thread.artifacts.statusReady")
  }
  if (status === "failed") return t("agentChat.thread.artifacts.statusLimited")
  return t("agentChat.thread.artifacts.statusSynced")
}

function readArtifactState(status: string): InlineAgentState {
  if (status === "active" || status === "completed" || status === "success") {
    return "done"
  }
  if (status === "failed") return "failed"
  return "working"
}

function sanitizeArtifactText(value: string | undefined, fallback: string) {
  const text = String(value ?? "").trim()
  if (!text || isInternalRuntimeText(text)) return fallback
  return text
}

function isSearchEvidenceArtifact(type: string | undefined) {
  return String(type ?? "").trim().toLowerCase() === "search_evidence"
}

function isMachineArtifactType(type: string) {
  return /^[a-z0-9]+(?:[_-][a-z0-9]+)+$/.test(type)
}
