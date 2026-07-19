import { describe, expect, it, vi } from "vitest"
import { agentChatData } from "../agent-chat-data"
import { uploadAgentChatAttachments } from "./agent-chat-attachments"
import type { AgentChatAttachmentItem } from "../types"

vi.mock("../agent-chat-data", () => ({
  AGENT_ID: "web-agent",
  agentChatData: {
    registerConversationFile: vi.fn(),
    updateConversationFileExtraction: vi.fn(),
    uploadDocument: vi.fn(),
  },
}))

describe("uploadAgentChatAttachments", () => {
  it("returns uploaded conversation file ids so new files can be selected for this turn", async () => {
    const attachments: AgentChatAttachmentItem[] = []
    vi.mocked(agentChatData.uploadDocument).mockResolvedValue({
      already_uploaded: false,
      key: "users/user-scope/docs/file-sha.txt",
      sha256: "file-sha",
      size: 12,
      url: "https://upload.test/file-sha.txt",
    })
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
      objectKey: "users/user-scope/docs/file-sha.txt",
      sha256: "file-sha",
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
      objectKey: "users/user-scope/docs/file-sha.txt",
      sha256: "file-sha",
      status: "active",
    })

    const result = await uploadAgentChatAttachments({
      conversationId: "conversation-upload",
      ensureConversation: vi.fn(),
      fallbackConversationTitle: "New chat",
      files: [new File(["hello upload"], "note.txt", { type: "text/plain" })],
      setAttachments: (updater) => {
        if (typeof updater === "function") {
          attachments.splice(0, attachments.length, ...updater(attachments))
        }
      },
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
