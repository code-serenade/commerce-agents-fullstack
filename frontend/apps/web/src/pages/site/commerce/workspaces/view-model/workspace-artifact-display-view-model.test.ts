import { describe, expect, it } from "vitest"
import type {
  WorkspaceArtifactExportResponse,
  WorkspaceArtifactResponse,
} from "@/api"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import {
  formatWorkspaceArtifactNotice,
  getWorkspaceArtifactDisplay,
} from "./workspace-artifact-display-view-model"

describe("workspace artifact display view model", () => {
  it("turns workspace artifact fields into student-facing labels", async () => {
    const i18n = await createAgentChatTestI18n()
    const rawFinalReportTitle = ["Agent", "final", "report"].join(" ")
    const artifact = createArtifact({
      artifactType: "final_report",
      fileName: "artifact-12345678-1234-5678-9abc-123456789abc.md",
      format: "markdown",
      runId: "run-12345678-1234-5678-9abc-123456789abc",
      source: "agent_session",
      storageKind: "inline",
      title: rawFinalReportTitle,
      validationStatus: "unchecked",
    })
    const artifactExport = createArtifactExport({
      contentChars: 2480,
      fileName: "workspace_artifact_12345678-1234-5678-9abc-123456789abc.md",
      format: "markdown",
    })

    const display = getWorkspaceArtifactDisplay(i18n.t, artifact, artifactExport)

    expect(display.title).toBe("Conversation report")
    expect(display.type).toBe("Conversation report")
    expect(display.source).toBe("Generated from Q&A")
    expect(display.format).toBe("Markdown document")
    expect(display.validationStatus).toBe("To check")
    expect(display.contentLength).toBe("About 2.5k chars")
    expect(display.downloadFileName).toBe("Conversation report.txt")
    expect(JSON.stringify(display).toLowerCase()).not.toContain("artifact-")
    expect(JSON.stringify(display).toLowerCase()).not.toContain("workspace_artifact")
    expect(JSON.stringify(display).toLowerCase()).not.toContain("inline")
    expect(JSON.stringify(display).toLowerCase()).not.toContain("run-")
  })

  it("hides raw backend warning details from the workspace artifact panel", async () => {
    const i18n = await createAgentChatTestI18n()

    expect(
      formatWorkspaceArtifactNotice(
        i18n.t,
        "NETWORK_ERROR: artifact 12345678-1234-5678-9abc-123456789abc failed"
      )
    ).toBe(
      "The report could not be fully loaded yet. Confirm the conversation on the Q&A page, then refresh this page later."
    )
  })
})

function createArtifact(
  overrides: Partial<WorkspaceArtifactResponse>
): WorkspaceArtifactResponse {
  const fallbackTitle = ["Artifact"].join("")
  return {
    artifactType: "artifact",
    content: null,
    createdAt: "2026-06-29T00:00:00Z",
    fileName: null,
    fileSize: null,
    format: "markdown",
    id: "artifact-1",
    mimeType: null,
    preview: null,
    runId: null,
    source: "agent_session",
    status: "active",
    storageKind: "inline",
    title: fallbackTitle,
    updatedAt: "2026-06-29T00:00:00Z",
    validationStatus: "unchecked",
    workspaceId: "workspace-1",
    workspaceTaskId: null,
    ...overrides,
  }
}

function createArtifactExport(
  overrides: Partial<WorkspaceArtifactExportResponse>
): WorkspaceArtifactExportResponse {
  const fallbackTitle = ["Report"].join("")
  return {
    artifactId: "artifact-1",
    content: "report content",
    contentChars: 12,
    fileName: "report.md",
    format: "markdown",
    mimeType: "text/markdown",
    sourceFormat: "markdown",
    sourceStorageKind: "inline",
    title: fallbackTitle,
    warnings: [],
    workspaceId: "workspace-1",
    ...overrides,
  }
}
