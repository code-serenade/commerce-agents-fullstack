import { createElement, type Dispatch, type ReactNode, type SetStateAction } from "react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { act, renderHook } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { agentChatData } from "../agent-chat-data"
import type { AgentChatAttachmentItem } from "../types"
import { useAgentChatSourceActions } from "./use-agent-chat-source-actions"

vi.mock("../agent-chat-data", () => ({
  AGENT_ID: "web-agent",
  agentChatData: {
    attachConversationFile: vi.fn(),
  },
}))

describe("useAgentChatSourceActions", () => {
  it("selects a library file for the current turn after attaching it", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    })
    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries")
    const restoreSelectedSources = vi.fn()
    const setAttachments = createAttachmentSetter()
    const existingTitle = ["existing", "txt"].join(".")

    vi.mocked(agentChatData.attachConversationFile).mockResolvedValue({
      bucket: "private",
      conversationId: "conversation-current",
      createdAt: "2026-05-27T08:00:00Z",
      fileName: "library.pdf",
      fileSize: 2048,
      id: "file-attached",
      messageId: null,
      metadata: { extractionStatus: "extracted" },
      mimeType: "application/pdf",
      objectKey: "docs/library.pdf",
      sha256: null,
      status: "active",
    })

    const { result } = renderHook(
      () =>
        useAgentChatSourceActions({
          conversationId: "conversation-current",
          ensureConversation: vi.fn(),
          fallbackConversationTitle: "New chat",
          isLoggedIn: true,
          isStreaming: false,
          restoreSelectedSources,
          selectedSources: [
            {
              id: "file-existing",
              kind: "conversation_file",
              readable: true,
              ref: "conversation_file:file-existing",
              status: "uploaded",
              title: existingTitle,
            },
          ],
          sessionIdRef: { current: "conversation-current" },
          setAttachments,
          setError: vi.fn(),
        }),
      {
        wrapper: ({ children }: { children: ReactNode }) =>
          createElement(QueryClientProvider, { client: queryClient }, children),
      }
    )

    await act(async () => {
      await result.current.attachExistingSource("file-library")
    })

    expect(agentChatData.attachConversationFile).toHaveBeenCalledWith(
      "file-library",
      { conversationId: "conversation-current" }
    )
    expect(setAttachments.items).toHaveLength(1)
    expect(setAttachments.items[0]).toMatchObject({
      fileId: "file-attached",
      name: "library.pdf",
      readable: true,
      status: "uploaded",
    })
    expect(restoreSelectedSources).toHaveBeenCalledWith([
      "conversation_file:file-existing",
      "conversation_file:file-attached",
    ])
    expect(invalidateQueries).toHaveBeenCalledWith({
      queryKey: ["agent-chat", "source-library"],
    })
    expect(invalidateQueries).toHaveBeenCalledWith({
      queryKey: ["commerce", "files"],
    })
  })
})

type AttachmentSetter = Dispatch<SetStateAction<AgentChatAttachmentItem[]>> & {
  items: AgentChatAttachmentItem[]
}

function createAttachmentSetter(): AttachmentSetter {
  const setAttachments = ((
    updater:
      | AgentChatAttachmentItem[]
      | ((current: AgentChatAttachmentItem[]) => AgentChatAttachmentItem[])
  ) => {
    setAttachments.items =
      typeof updater === "function" ? updater(setAttachments.items) : updater
  }) as AttachmentSetter
  setAttachments.items = [] as AgentChatAttachmentItem[]
  return setAttachments
}
