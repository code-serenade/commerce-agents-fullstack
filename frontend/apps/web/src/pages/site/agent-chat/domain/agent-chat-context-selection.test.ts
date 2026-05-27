import { describe, expect, it } from "vitest"
import {
  createContextRefs,
  mapConversationFileToSourceLibraryItem,
  mapCampaignToContextSelection,
  mapAttachmentToSourceContextSelection,
  restoreCampaignContextSelection,
  restoreSourceContextSelection,
  toggleCampaignContextSelection,
  toggleSourceContextSelection,
  type AgentChatSourceContextSelection,
  type AgentChatProductContextSelection,
  type AgentChatCampaignContextSelection,
} from "./agent-chat-context-selection"

const sourceTitle = "brief.md"
const imageSourceTitle = "shelf.png"
const pendingSourceTitle = "dataset.bin"
const librarySourceTitle = "training.pdf"

describe("agent chat context selection", () => {
  it("combines product, campaign, and explicit source refs without duplicates", () => {
    const products: AgentChatProductContextSelection[] = [
      { id: "product-a", name: "Product A" },
    ]
    const campaign: AgentChatCampaignContextSelection = {
      id: "campaign-a",
      name: "Campaign A",
    }
    const source: AgentChatSourceContextSelection = {
      id: "file-a",
      kind: "conversation_file" as const,
      readable: true,
      ref: "conversation_file:file-a",
      status: "uploaded" as const,
      title: sourceTitle,
    }

    expect(createContextRefs(products, [source, source], [campaign, campaign])).toEqual([
      "product:product-a",
      "conversation_file:file-a",
      "campaign:campaign-a",
    ])
  })

  it("maps, toggles, and restores campaign selections", () => {
    const campaign = mapCampaignToContextSelection({
      budget: 3000,
      campaignType: "launch",
      id: "campaign-a",
      name: "Launch A",
      productId: "product-a",
      status: "active",
    })

    expect(campaign).toEqual(
      expect.objectContaining({
        budget: 3000,
        campaignType: "launch",
        id: "campaign-a",
        name: "Launch A",
      })
    )
    expect(toggleCampaignContextSelection([], campaign)).toEqual([campaign])
    expect(toggleCampaignContextSelection([campaign], campaign)).toEqual([])
    expect(
      restoreCampaignContextSelection([campaign], ["campaign-a", "missing"])
    ).toEqual([campaign])
  })

  it("maps registered conversation files into selectable source refs", () => {
    expect(
      mapAttachmentToSourceContextSelection({
        bucket: "private",
        fileId: "file-a",
        id: "local-a",
        name: sourceTitle,
        objectKey: "users/u/docs/file-a.md",
        readable: true,
        status: "uploaded",
      })
    ).toEqual({
      bucket: "private",
      id: "file-a",
      kind: "conversation_file",
      objectKey: "users/u/docs/file-a.md",
      readable: true,
      ref: "conversation_file:file-a",
      status: "uploaded",
      title: sourceTitle,
      visionReady: false,
    })

    expect(
      mapAttachmentToSourceContextSelection({
        id: "local-b",
        name: "pending.pdf",
        readable: false,
        status: "uploading",
      })
    ).toBeNull()

    expect(
      mapAttachmentToSourceContextSelection({
        bucket: "private",
        fileId: "file-pending",
        id: "local-pending",
        name: pendingSourceTitle,
        objectKey: "users/u/docs/file-pending.bin",
        readable: false,
        status: "pending",
      })
    ).toEqual({
      bucket: "private",
      id: "file-pending",
      kind: "conversation_file",
      objectKey: "users/u/docs/file-pending.bin",
      readable: false,
      ref: "conversation_file:file-pending",
      status: "pending",
      title: pendingSourceTitle,
      visionReady: false,
    })
  })

  it("maps historical files into source library items with attached state", () => {
    const libraryItem = mapConversationFileToSourceLibraryItem(
      {
        bucket: "private",
        conversationId: "conversation-old",
        createdAt: "2026-05-26T08:00:00Z",
        fileName: librarySourceTitle,
        fileSize: 2048,
        id: "file-library",
        messageId: null,
        metadata: { extractionStatus: "extracted" },
        mimeType: "application/pdf",
        objectKey: "users/u/docs/training.pdf",
        sha256: null,
        status: "active",
      },
      new Set(["private:users/u/docs/training.pdf"])
    )

    expect(libraryItem).toEqual({
      attached: true,
      bucket: "private",
      createdAt: "2026-05-26T08:00:00Z",
      fileSize: 2048,
      id: "file-library",
      mimeType: "application/pdf",
      objectKey: "users/u/docs/training.pdf",
      readable: true,
      status: "readable",
      title: librarySourceTitle,
      visionReady: false,
    })
  })

  it("marks uploaded vision images as selectable source refs", () => {
    expect(
      mapAttachmentToSourceContextSelection({
        bucket: "private",
        fileId: "file-image",
        id: "local-image",
        imageDataUrl: "data:image/png;base64,aW1hZ2U=",
        kind: "image",
        name: imageSourceTitle,
        objectKey: "users/u/images/file-image.png",
        readable: true,
        status: "uploaded",
      })
    ).toEqual({
      bucket: "private",
      id: "file-image",
      kind: "conversation_file",
      objectKey: "users/u/images/file-image.png",
      readable: true,
      ref: "conversation_file:file-image",
      status: "uploaded",
      title: imageSourceTitle,
      visionReady: true,
    })
  })

  it("toggles explicit source selections by ref", () => {
    const source: AgentChatSourceContextSelection = {
      id: "file-a",
      kind: "conversation_file" as const,
      readable: true,
      ref: "conversation_file:file-a",
      status: "uploaded" as const,
      title: sourceTitle,
    }

    expect(toggleSourceContextSelection([], source)).toEqual([source])
    expect(toggleSourceContextSelection([source], source)).toEqual([])
  })

  it("restores only available conversation file selections", () => {
    const source: AgentChatSourceContextSelection = {
      id: "file-a",
      kind: "conversation_file" as const,
      readable: true,
      ref: "conversation_file:file-a",
      status: "uploaded" as const,
      title: sourceTitle,
    }

    expect(
      restoreSourceContextSelection(
        [source],
        ["product:product-a", "conversation_file:file-a", "conversation_file:missing"]
      )
    ).toEqual([source])
  })
})
