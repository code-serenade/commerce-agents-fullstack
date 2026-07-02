import type { TFunction } from "i18next"
import type { WorkspaceArtifactResponse } from "@/api"

export interface OutputArtifactDisplay {
  downloadBaseName: string
  format: string
  source: string
  status: string
  title: string
  type: string
  validationStatus: string
}

export function getOutputArtifactDisplay(
  t: TFunction,
  artifact: WorkspaceArtifactResponse | null | undefined
): OutputArtifactDisplay {
  const rawTitle = artifact?.title?.trim() ?? ""

  return {
    downloadBaseName: formatDownloadBaseName(t, rawTitle, artifact?.artifactType),
    format: formatArtifactFormat(t, artifact?.format),
    source: formatArtifactSource(t, artifact?.source),
    status: formatArtifactStatus(t, artifact?.status),
    title: formatArtifactTitle(t, rawTitle, artifact?.artifactType),
    type: formatArtifactType(t, artifact?.artifactType),
    validationStatus: formatArtifactValidationStatus(t, artifact?.validationStatus),
  }
}

export function formatArtifactFormat(t: TFunction, format?: string) {
  const normalized = String(format ?? "").trim().toLowerCase()
  if (normalized === "markdown") return t("agentOutput.backend.display.format.markdown")
  if (normalized === "json") return t("agentOutput.backend.display.format.json")
  if (normalized === "docx") return t("agentOutput.backend.display.format.docx")
  if (normalized === "pdf") return "PDF"
  if (normalized === "txt" || normalized === "text") {
    return t("agentOutput.backend.display.format.text")
  }
  return normalized ? normalized.toUpperCase() : "-"
}

export function formatStorageKind(t: TFunction, storageKind?: string) {
  const normalized = String(storageKind ?? "").trim().toLowerCase()
  if (normalized === "inline") return t("agentOutput.backend.display.storage.inline")
  if (normalized === "file") return t("agentOutput.backend.display.storage.file")
  return normalized || "-"
}

export function formatContentLength(t: TFunction, value?: number | null) {
  if (!Number.isFinite(value ?? Number.NaN) || !value || value <= 0) return "-"
  if (value < 1000) {
    return t("agentOutput.backend.display.contentLength.chars", {
      count: Math.round(value),
    })
  }
  return t("agentOutput.backend.display.contentLength.thousands", {
    count: Math.round(value / 100) / 10,
  })
}

export function formatShortId(value?: string | null) {
  const text = value?.trim()
  if (!text) return "-"
  if (text.length <= 12) return text
  return `${text.slice(0, 8)}...${text.slice(-4)}`
}

export function formatOutputBackendNotice(t: TFunction, detail?: string | null) {
  const text = String(detail ?? "").trim()
  if (!text) return t("agentOutput.backend.display.notice.generic")
  if (isRawBackendDetail(text)) {
    return t("agentOutput.backend.display.notice.technical")
  }
  return text
}

function isRawBackendDetail(text: string) {
  return (
    /^[A-Z_]+:/.test(text) ||
    /\b(fetch|network|timeout|failed|error|exception|stack|undefined|null)\b/i.test(text) ||
    /\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b/i.test(text) ||
    /[[\]{}"`]/.test(text)
  )
}

function formatArtifactTitle(t: TFunction, title: string, artifactType?: string) {
  if (!title || title === "Agent final report") {
    return formatArtifactType(t, artifactType)
  }

  return title
    .replace(/\bAgent final report\b/gi, t("agentOutput.backend.display.type.finalReport"))
    .replace(/\bfinal report\b/gi, t("agentOutput.backend.display.type.report"))
}

function formatDownloadBaseName(t: TFunction, title: string, artifactType?: string) {
  return sanitizeFileBaseName(
    formatArtifactTitle(t, title, artifactType),
    t("agentOutput.backend.downloadFileName")
  )
}

function sanitizeFileBaseName(value: string, fallback: string) {
  return (
    value
      .trim()
      .replace(/[\\/:*?"<>|]+/g, "-")
      .replace(/\s+/g, " ")
      .slice(0, 48) || fallback
  )
}

function formatArtifactType(t: TFunction, type?: string) {
  const normalized = String(type ?? "").trim().toLowerCase()
  if (normalized === "final_report") return t("agentOutput.backend.display.type.finalReport")
  if (normalized === "search_evidence") {
    return t("agentOutput.backend.display.type.searchEvidence")
  }
  if (normalized === "markdown") return t("agentOutput.backend.display.format.markdown")
  if (!normalized || normalized === "artifact") {
    return t("agentOutput.backend.display.type.learningMaterial")
  }
  return normalized.replaceAll("_", " ")
}

function formatArtifactStatus(t: TFunction, status?: string) {
  const normalized = String(status ?? "").trim().toLowerCase()
  if (normalized === "active") return t("agentOutput.backend.display.status.active")
  if (normalized === "failed") return t("agentOutput.backend.display.status.failed")
  if (normalized === "pending") return t("agentOutput.backend.display.status.pending")
  if (normalized === "archived") return t("agentOutput.backend.display.status.archived")
  return normalized || "-"
}

function formatArtifactValidationStatus(t: TFunction, status?: string) {
  const normalized = String(status ?? "").trim().toLowerCase()
  if (normalized === "unchecked") return t("agentOutput.backend.display.validation.unchecked")
  if (normalized === "passed") return t("agentOutput.backend.display.validation.passed")
  if (normalized === "failed") return t("agentOutput.backend.display.validation.failed")
  return normalized || "-"
}

function formatArtifactSource(t: TFunction, source?: string) {
  const normalized = String(source ?? "").trim().toLowerCase()
  if (normalized === "agent_session") return t("agentOutput.backend.display.source.agentSession")
  if (normalized === "agent_skill_result") {
    return t("agentOutput.backend.display.source.agentSkillResult")
  }
  return normalized ? normalized.replaceAll("_", " ") : "-"
}
