import type { TFunction } from "i18next"
import type {
  WorkspaceArtifactExportResponse,
  WorkspaceArtifactResponse,
} from "@/api"
import {
  formatArtifactFormat,
  formatContentLength,
  formatOutputBackendNotice,
  getOutputArtifactDisplay,
} from "@/pages/site/agent-output/view-model/output-artifact-display-view-model"

export interface WorkspaceArtifactDisplay {
  contentLength: string
  downloadFileName: string
  format: string
  source: string
  status: string
  title: string
  type: string
  validationStatus: string
}

export function getWorkspaceArtifactDisplay(
  t: TFunction,
  artifact: WorkspaceArtifactResponse | null | undefined,
  artifactExport?: WorkspaceArtifactExportResponse | null
): WorkspaceArtifactDisplay {
  const display = getOutputArtifactDisplay(t, artifact)
  const rawFileName = artifactExport?.fileName?.trim() || artifact?.fileName?.trim()
  const downloadFileName =
    rawFileName && !looksLikeInternalFileName(rawFileName)
      ? rawFileName
      : `${display.downloadBaseName}.txt`

  return {
    contentLength: formatContentLength(
      t,
      artifactExport?.contentChars ??
        artifact?.content?.length ??
        artifact?.preview?.length ??
        null
    ),
    downloadFileName,
    format: formatArtifactFormat(t, artifactExport?.format || artifact?.format),
    source: display.source,
    status: display.status,
    title: display.title,
    type: display.type,
    validationStatus: display.validationStatus,
  }
}

export function formatWorkspaceArtifactNotice(
  t: TFunction,
  detail?: string | null
) {
  return formatOutputBackendNotice(t, detail)
}

function looksLikeInternalFileName(value: string) {
  return (
    /\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b/i.test(
      value
    ) ||
    /^artifact[-_]/i.test(value) ||
    /workspace[_-]?artifact/i.test(value)
  )
}
