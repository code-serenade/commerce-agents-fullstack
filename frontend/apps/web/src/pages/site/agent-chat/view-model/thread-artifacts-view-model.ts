import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { InlineAgentState } from "./thread-inline-view-model"

export interface InlineArtifact {
  filename: string
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
      filename: readArtifactFilename(event),
      meta: readArtifactMeta(event),
      nodeId: "delivery",
      status: readArtifactStatus(event, t),
      state: readArtifactState(event.status),
    }))
}

function readArtifactFilename(
  event: Extract<AgentRuntimeEvent, { type: "workspace_artifact_created" }>
) {
  return event.fileName ?? event.file_name ?? event.title
}

function readArtifactMeta(
  event: Extract<AgentRuntimeEvent, { type: "workspace_artifact_created" }>
) {
  return (
    event.artifactType ??
    event.artifact_type ??
    event.format ??
    event.storageKind ??
    event.storage_kind ??
    "artifact"
  )
}

function readArtifactStatus(
  event: Extract<AgentRuntimeEvent, { type: "workspace_artifact_created" }>,
  t: TFunction
) {
  const artifactType = event.artifactType ?? event.artifact_type ?? "artifact"
  const preview = event.preview?.trim()
  if (artifactType === "search_evidence") {
    return t("agentChat.thread.artifacts.searchEvidence")
  }
  if (preview) return preview
  return t("agentChat.thread.artifacts.backendCreated", {
    type: artifactType,
    status: event.status,
  })
}

function readArtifactState(status: string): InlineAgentState {
  if (status === "active" || status === "completed" || status === "success") {
    return "done"
  }
  if (status === "failed") return "failed"
  return "working"
}
