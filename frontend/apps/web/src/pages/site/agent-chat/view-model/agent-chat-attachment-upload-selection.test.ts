import { describe, expect, it, vi } from "vitest"
import { agentChatData } from "../agent-chat-data"
import { uploadAgentChatAttachments } from "./agent-chat-attachments"
import type { AgentChatAttachmentItem } from "../types"

vi.mock("../agent-chat-data", () => ({
  AGENT_ID: "web-agent",
  agentChatData: {
    getUploadDocumentSign: vi.fn(),
    registerConversationFile: vi.fn(),
    updateConversationFileExtraction: vi.fn(),
    uploadWithSignedUrl: vi.fn(),
  },
}))

describe("uploadAgentChatAttachments", () => {
  it("returns uploaded conversation file ids so new files can be selected for this turn", async () => {
    const attachments: AgentChatAttachmentItem[] = []
    vi.mocked(agentChatData.getUploadDocumentSign).mockResolvedValue({
      headers: {
        "Content-Type": "text/plain",
        authorization: "signed",
        "x-amz-content-sha256": "unsigned-payload",
        "x-amz-date": "20260526T080000Z",
      },
      key: "chat/user/conversation/note.txt",
      method: "PUT",
      upload_url: "https://upload.test/note.txt",
      already_uploaded: false,
    })
    vi.mocked(agentChatData.uploadWithSignedUrl).mockResolvedValue(
      new Response(null, { status: 200 })
    )
    vi.mocked(agentChatData.registerConversationFile).mockResolvedValue({
      bucket: "private",
      conversationId: "conversation-upload",
      createdAt: "2026-05-26T08:00:00Z",
      fileName: "note.txt",
      fileSize: 12,
      id: "file-uploaded",
      messageId: null,
      metadata: {},
      mimeType: "text/plain",
      objectKey: "chat/user/conversation/note.txt",
      sha256: null,
      status: "active",
    })
    vi.mocked(agentChatData.updateConversationFileExtraction).mockResolvedValue({
      bucket: "private",
      conversationId: "conversation-upload",
      createdAt: "2026-05-26T08:00:00Z",
      fileName: "note.txt",
      fileSize: 12,
      id: "file-uploaded",
      messageId: null,
      metadata: { extractionStatus: "extracted" },
      mimeType: "text/plain",
      objectKey: "chat/user/conversation/note.txt",
      sha256: null,
      status: "active",
    })

    const result = await uploadAgentChatAttachments({
      conversationId: "conversation-upload",
      fallbackConversationTitle: "New chat",
      files: [new File(["hello upload"], "note.txt", { type: "text/plain" })],
      setAttachments: (updater) => {
        if (typeof updater === "function") {
          attachments.splice(0, attachments.length, ...updater(attachments))
        }
      },
      setConversationId: vi.fn(),
      setConversations: vi.fn(),
    })

    expect(result).toEqual({
      conversationId: "conversation-upload",
      fileIds: ["file-uploaded"],
    })
    expect(attachments[0]).toMatchObject({
      fileId: "file-uploaded",
      readable: true,
      status: "uploaded",
    })
  })
})
