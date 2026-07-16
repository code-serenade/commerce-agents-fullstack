import type {
  WorkspaceArtifactExportResponse,
  WorkspaceArtifactResponse,
} from "@/api"
import type { AgentOutputReport } from "../types"
import { parseMarkdownReport } from "./report-markdown"
import {
  firstText,
  firstTextOrNull,
  mergeRecords,
  parseJsonRecord,
  readText,
  safeFileName,
  uniqueTexts,
} from "./report-parser-utils"
import { findStructuredRecord, readStructuredReport } from "./report-structured"

export function buildAgentOutputReport(
  artifact: WorkspaceArtifactResponse | null,
  artifactExport: WorkspaceArtifactExportResponse | null
): AgentOutputReport | null {
  if (!artifact) return null

  const content = firstText(
    artifactExport?.content,
    artifact.content,
    artifact.preview
  )
  const metadata = mergeRecords(artifact.metadata, artifactExport?.metadata)
  const contentRecord = parseJsonRecord(content)
  const structuredRecord = findStructuredRecord(metadata, contentRecord)
  const structured = readStructuredReport(structuredRecord)
  const markdown = parseMarkdownReport(content)
  const metrics = structured.metrics ?? markdown.metrics
  const steps = structured.steps ?? markdown.steps
  const actions = structured.actions ?? markdown.actions
  const sections = structured.sections ?? markdown.sections
  const title = firstText(structured.title, markdown.title, artifact.title)
  const description = firstText(
    structured.description,
    markdown.description,
    artifact.preview,
    sections[0]?.content
  )

  if (!content && !sections.length && !description) return null

  return {
    actions,
    badges: uniqueTexts(structured.badges).slice(0, 3),
    collaboration: firstTextOrNull(
      structured.collaboration,
      readText(metadata, "collaboration", "dispatch", "agents", "agentRouting")
    ),
    content,
    description,
    fileName:
      artifactExport?.fileName ||
      artifact.fileName ||
      `${safeFileName(title)}.txt`,
    metrics,
    prompt: firstTextOrNull(
      structured.prompt,
      readText(metadata, "prompt", "userPrompt", "question", "input")
    ),
    sections: sections.length
      ? sections
      : description
        ? [
            {
              id: "overview",
              title,
              eyebrow: "",
              content: description,
              bullets: [],
            },
          ]
        : [],
    steps,
    title,
  }
}
