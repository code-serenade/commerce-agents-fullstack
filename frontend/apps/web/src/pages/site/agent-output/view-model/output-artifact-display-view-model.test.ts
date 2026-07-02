import { describe, expect, it } from "vitest"
import type { AgentOutputDeliverable } from "../use-agent-output-page"
import {
  formatArtifactFormat,
  formatContentLength,
  formatOutputBackendNotice,
  formatShortId,
  formatStorageKind,
  getOutputArtifactDisplay,
} from "./output-artifact-display-view-model"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"

describe("output artifact display view model", () => {
  it("turns backend artifact fields into user-facing labels", async () => {
    const i18n = await createAgentChatTestI18n()
    const rawReportTitle = "Agent final report"
    const reportTitle = "Conversation report"
    const artifact = createArtifact({
      artifactType: "final_report",
      format: "markdown",
      source: "agent_session",
      status: "active",
      title: rawReportTitle,
      validationStatus: "unchecked",
    })

    expect(getOutputArtifactDisplay(i18n.t, artifact)).toEqual({
      downloadBaseName: reportTitle,
      format: "Markdown document",
      source: "Generated from Q&A",
      status: "Ready to view",
      title: reportTitle,
      type: reportTitle,
      validationStatus: "To check",
    })
  })

  it("shortens long ids and formats storage without exposing raw values first", async () => {
    const i18n = await createAgentChatTestI18n()

    expect(formatShortId("12345678-1234-5678-9abc-123456789abc")).toBe(
      "12345678...9abc"
    )
    expect(formatStorageKind(i18n.t, "inline")).toBe("Saved body")
    expect(formatArtifactFormat(i18n.t, "docx")).toBe("Word document")
  })

  it("formats content length as readable copy instead of raw bytes", async () => {
    const i18n = await createAgentChatTestI18n()

    expect(formatContentLength(i18n.t, 460)).toBe("About 460 chars")
    expect(formatContentLength(i18n.t, 2480)).toBe("About 2.5k chars")
    expect(formatContentLength(i18n.t, null)).toBe("-")
  })

  it("hides raw backend errors from the report page primary notice", async () => {
    const i18n = await createAgentChatTestI18n()
    const rawDetail =
      "NETWORK_ERROR: artifact 12345678-1234-5678-9abc-123456789abc failed"

    expect(formatOutputBackendNotice(i18n.t, rawDetail)).toBe(
      "The report could not be fully loaded yet. Confirm the conversation on the Q&A page, then refresh this page later."
    )
    expect(formatOutputBackendNotice(i18n.t, "Please refresh later")).toBe(
      "Please refresh later"
    )
  })
})

function createArtifact(
  overrides: Partial<AgentOutputDeliverable>
): AgentOutputDeliverable {
  const title = "Artifact"
  const workspaceTitle = "Training task"

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
    title,
    updatedAt: "2026-06-29T00:00:00Z",
    validationStatus: "unchecked",
    workspace: {
      id: "workspace-1",
      status: "active",
      title: workspaceTitle,
    },
    workspaceId: "workspace-1",
    workspaceTaskId: null,
    ...overrides,
  }
}
