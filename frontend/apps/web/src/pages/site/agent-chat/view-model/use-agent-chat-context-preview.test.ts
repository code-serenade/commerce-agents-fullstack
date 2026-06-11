import { act, renderHook, waitFor } from "@testing-library/react"
import { beforeEach, describe, expect, it, vi } from "vitest"
import { agentChatData } from "../agent-chat-data"
import { useAgentChatContextPreview } from "./use-agent-chat-context-preview"

const sourceTitle = "brief.pdf"

vi.mock("../agent-chat-data", () => ({
  agentChatData: {
    previewAgentContext: vi.fn(),
  },
}))

describe("useAgentChatContextPreview", () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it("calls the preview API with the same context refs used by sending", async () => {
    vi.mocked(agentChatData.previewAgentContext).mockResolvedValue({
      budget: {},
      includedRefs: ["product:p1"],
      memoryRecords: [],
      metadataOnlyRefs: [],
      omittedRefs: [],
      warnings: [],
      workspaceContext: {},
      workspacePack: {},
    } as Awaited<ReturnType<typeof agentChatData.previewAgentContext>>)

    const { result } = renderHook(() =>
      useAgentChatContextPreview({
        conversationId: "conversation-1",
        inputValue: "  analyze this turn  ",
        isLoggedIn: true,
        isStreaming: false,
        selectedCampaigns: [{ id: "c1", name: "campaign_fixture" }],
        selectedProducts: [{ id: "p1", name: "product_fixture" }],
        selectedSources: [
          {
            id: "f1",
            kind: "conversation_file",
            readable: true,
            ref: "conversation_file:f1",
            status: "uploaded",
            title: sourceTitle,
          },
        ],
        tError: "preview failed",
      })
    )

    await act(async () => {
      await result.current.previewContext()
    })

    expect(agentChatData.previewAgentContext).toHaveBeenCalledWith({
      contextRefs: ["product:p1", "conversation_file:f1", "campaign:c1"],
      conversationId: "conversation-1",
      input: "analyze this turn",
      productId: "p1",
    })
    expect(result.current.previewContextResult?.includedRefs).toEqual([
      "product:p1",
    ])
  })

  it("does not call the preview API without a sendable prompt", async () => {
    const { result } = renderHook(() =>
      useAgentChatContextPreview({
        conversationId: null,
        inputValue: "  ",
        isLoggedIn: true,
        isStreaming: false,
        selectedCampaigns: [],
        selectedProducts: [],
        selectedSources: [],
        tError: "preview failed",
      })
    )

    await act(async () => {
      await result.current.previewContext()
    })

    expect(agentChatData.previewAgentContext).not.toHaveBeenCalled()
  })

  it("clears stale preview results when the prompt changes", async () => {
    vi.mocked(agentChatData.previewAgentContext).mockResolvedValue({
      budget: {},
      includedRefs: [],
      memoryRecords: [],
      metadataOnlyRefs: [],
      omittedRefs: [],
      warnings: [],
      workspaceContext: {},
      workspacePack: {},
    } as Awaited<ReturnType<typeof agentChatData.previewAgentContext>>)

    const { rerender, result } = renderHook(
      ({ inputValue }) =>
        useAgentChatContextPreview({
          conversationId: null,
          inputValue,
          isLoggedIn: true,
          isStreaming: false,
          selectedCampaigns: [],
          selectedProducts: [],
          selectedSources: [],
          tError: "preview failed",
        }),
      { initialProps: { inputValue: "first prompt" } }
    )

    await act(async () => {
      await result.current.previewContext()
    })
    expect(result.current.previewContextResult).not.toBeNull()

    rerender({ inputValue: "second prompt" })

    await waitFor(() => {
      expect(result.current.previewContextResult).toBeNull()
    })
  })
})
