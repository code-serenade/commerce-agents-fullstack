import { describe, expect, it } from "vitest"
import type { AgentOutputDeliverable } from "../agent-output-backend-data"
import {
  createConversationReportRequest,
  defaultConversationReportDraft,
  getConversationReportContext,
  isConversationReportArtifact,
} from "./conversation-report-view-model"

describe("conversation report view model", () => {
  it("prefers explicit URL context over selected artifact metadata", () => {
    const params = new URLSearchParams({
      conversationId: "conversation-url",
      runId: "run-url",
      workspaceId: "workspace-url",
    })
    const artifact = createArtifact({
      metadata: {
        conversationId: "conversation-meta",
        runId: "run-meta",
        workspaceId: "workspace-meta",
      },
      runId: "run-artifact",
      workspaceId: "workspace-artifact",
    })

    expect(getConversationReportContext(params, artifact)).toEqual({
      conversationId: "conversation-url",
      runId: "run-url",
      workspaceId: "workspace-url",
    })
  })

  it("builds a manual markdown report request from the draft and context", () => {
    expect(
      createConversationReportRequest(defaultConversationReportDraft, {
        conversationId: "conversation-1",
        runId: "run-1",
        workspaceId: "workspace-1",
      })
    ).toEqual({
      format: "markdown",
      includeProcess: true,
      includeSources: true,
      includeTranscriptAppendix: false,
      reportType: "commerce_conversation_report",
      runId: "run-1",
      scope: "conversation_all",
      workspaceId: "workspace-1",
    })
  })

  it("matches artifacts that belong to the current conversation", () => {
    expect(
      isConversationReportArtifact(
        createArtifact({
          metadata: {
            conversationId: "conversation-current",
          },
        }),
        "conversation-current"
      )
    ).toBe(true)

    expect(
      isConversationReportArtifact(
        createArtifact({
          metadata: {
            conversationId: "conversation-other",
          },
        }),
        "conversation-current"
      )
    ).toBe(false)
  })
})

function createArtifact(
  overrides: Partial<AgentOutputDeliverable>
): AgentOutputDeliverable {
  const emptyTitle = ""

  return {
    artifactType: "final_report",
    content: null,
    createdAt: "2026-07-08T00:00:00Z",
    fileName: null,
    fileSize: null,
    format: "markdown",
    id: "artifact-1",
    metadata: {},
    mimeType: null,
    preview: null,
    runId: "run-default",
    source: "agent_session",
    status: "active",
    storageKind: "inline",
    title: emptyTitle,
    updatedAt: "2026-07-08T00:00:00Z",
    validationStatus: "unchecked",
    workspace: {
      id: "workspace-default",
      status: "active",
      title: emptyTitle,
    },
    workspaceId: "workspace-default",
    workspaceTaskId: null,
    ...overrides,
  }
}
