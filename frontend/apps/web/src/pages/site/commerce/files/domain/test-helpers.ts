import type { TrainingFileItem } from "../types"

export function file(overrides: Partial<TrainingFileItem>): TrainingFileItem {
  return {
    id: "file-1",
    conversationId: "conversation-1",
    messageId: null,
    bucket: "agent-files",
    objectKey: "chat/user/conversation/file.pdf",
    fileName: "file.pdf",
    mimeType: "application/pdf",
    fileSize: 2048,
    sha256: null,
    status: "active",
    metadata: {},
    createdAt: "2026-05-26T08:00:00Z",
    ...overrides,
  }
}
