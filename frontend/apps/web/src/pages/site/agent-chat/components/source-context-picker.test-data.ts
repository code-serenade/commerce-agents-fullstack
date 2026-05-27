import type {
  AgentChatSourceContextSelection,
  AgentChatSourceLibraryItem,
} from "../domain/agent-chat-context-selection"

export const sourcePickerTestTitles = {
  attached: "file-attached.pdf",
  brief: "brief.pdf",
  library: "file-library.pdf",
  libraryPrefix: "library",
  libraryQuery: "library-7",
  pending: "pending-material.docx",
  sales: "sales-data.csv",
  unsupported: "unsupported-material.xls",
}

export function createSourceSelection(
  overrides: Partial<AgentChatSourceContextSelection> & {
    id: string
    title: string
  }
): AgentChatSourceContextSelection {
  return {
    kind: "conversation_file",
    readable: true,
    ref: `conversation_file:${overrides.id}`,
    status: "uploaded",
    ...overrides,
  }
}

export function createIndexedLibraryItems(count: number) {
  return Array.from({ length: count }, (_, index) =>
    createLibraryItem({
      id: `file-${index}`,
      objectKey: `docs/file-${index}.pdf`,
      title: getIndexedLibraryTitle(index),
    })
  )
}

export function getIndexedLibraryTitle(index: number) {
  return `${sourcePickerTestTitles.libraryPrefix}-${index}.pdf`
}

export function createLibraryItem(
  overrides: Partial<AgentChatSourceLibraryItem> & {
    id: string
    title: string
  }
): AgentChatSourceLibraryItem {
  return {
    attached: false,
    bucket: "private",
    createdAt: "2026-05-26T08:00:00Z",
    fileSize: 1024,
    mimeType: "application/pdf",
    objectKey: `docs/${overrides.id}.pdf`,
    readable: true,
    status: "readable",
    visionReady: false,
    ...overrides,
  }
}
