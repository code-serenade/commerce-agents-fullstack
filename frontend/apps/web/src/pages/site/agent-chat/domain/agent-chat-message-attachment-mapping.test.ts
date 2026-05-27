import { describe, expect, it } from "vitest"
import type { ConversationDetailResponse } from "@/api"
import { mapConversationAttachments } from "./agent-chat-message-mapping"

const imageTitle = "shelf.png"

describe("agent chat conversation attachment mapping", () => {
  it("maps persisted image files with vision metadata into composer attachments", () => {
    const detail = conversationDetail()
    detail.files = [
      {
        bucket: "agent-files",
        conversationId: "c1",
        createdAt: "2026-05-13T00:00:00.000Z",
        fileName: imageTitle,
        fileSize: 120,
        id: "file-image",
        messageId: null,
        metadata: {
          extraction: {
            metadata: {
              imageDataUrl: "data:image/png;base64,aW1hZ2U=",
              visionReady: true,
            },
          },
          extractionStatus: "extracted",
        },
        mimeType: "image/png",
        objectKey: "conversation/file-image.png",
        sha256: null,
        status: "active",
      },
    ]

    expect(mapConversationAttachments(detail)).toEqual([
      {
        bucket: "agent-files",
        fileId: "file-image",
        id: "file-image",
        imageDataUrl: "data:image/png;base64,aW1hZ2U=",
        imageDetail: "auto",
        kind: "image",
        name: imageTitle,
        objectKey: "conversation/file-image.png",
        readable: true,
        status: "uploaded",
        unsupportedReason: undefined,
      },
    ])
  })

  it("maps pending and unsupported extraction status honestly", () => {
    const detail = conversationDetail()
    detail.files = [
      {
        bucket: "agent-files",
        conversationId: "c1",
        createdAt: "2026-05-13T00:00:00.000Z",
        fileName: "dataset.bin",
        fileSize: 120,
        id: "file-pending",
        messageId: null,
        metadata: {
          extraction: {
            status: "pending",
          },
          extractionStatus: "pending",
        },
        mimeType: "application/octet-stream",
        objectKey: "conversation/dataset.bin",
        sha256: null,
        status: "active",
      },
      {
        bucket: "agent-files",
        conversationId: "c1",
        createdAt: "2026-05-13T00:00:00.000Z",
        fileName: "brief.pdf",
        fileSize: 120,
        id: "file-unsupported",
        messageId: null,
        metadata: {
          extraction: {
            failureReason:
              "server object-body parser is not enabled for this file type yet",
            status: "unsupported",
          },
          extractionStatus: "unsupported",
        },
        mimeType: "application/pdf",
        objectKey: "conversation/brief.pdf",
        sha256: null,
        status: "active",
      },
    ]

    expect(mapConversationAttachments(detail)).toEqual([
      expect.objectContaining({
        fileId: "file-pending",
        readable: false,
        status: "pending",
      }),
      expect.objectContaining({
        fileId: "file-unsupported",
        readable: false,
        status: "unsupported",
        unsupportedReason:
          "server object-body parser is not enabled for this file type yet",
      }),
    ])
  })

  it("maps server extracted binary documents as readable attachments", () => {
    const detail = conversationDetail()
    detail.files = [
      {
        bucket: "agent-files",
        conversationId: "c1",
        createdAt: "2026-05-13T00:00:00.000Z",
        fileName: "brief.pdf",
        fileSize: 120,
        id: "file-pdf",
        messageId: null,
        metadata: {
          extraction: {
            status: "extracted",
          },
          extractionStatus: "extracted",
        },
        mimeType: "application/pdf",
        objectKey: "conversation/brief.pdf",
        sha256: null,
        status: "active",
      },
    ]

    expect(mapConversationAttachments(detail)).toEqual([
      expect.objectContaining({
        fileId: "file-pdf",
        readable: true,
        status: "uploaded",
      }),
    ])
  })
})

function conversationDetail(): ConversationDetailResponse {
  return {
    conversation: {
      agentId: "web-agent",
      createdAt: "2026-05-13T00:00:00.000Z",
      id: "c1",
      modelName: null,
      source: "agent_chat",
      status: "active",
      title: null,
      updatedAt: "2026-05-13T00:00:00.000Z",
      userId: "u1",
    },
    files: [],
    messages: [],
  }
}
