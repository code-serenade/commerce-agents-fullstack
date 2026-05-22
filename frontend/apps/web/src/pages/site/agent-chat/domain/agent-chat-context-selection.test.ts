import { describe, expect, it } from "vitest"
import {
  createContextRefs,
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
        fileId: "file-a",
        id: "local-a",
        name: sourceTitle,
        readable: true,
        status: "uploaded",
      })
    ).toEqual({
      id: "file-a",
      kind: "conversation_file",
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
        fileId: "file-pending",
        id: "local-pending",
        name: pendingSourceTitle,
        readable: false,
        status: "pending",
      })
    ).toEqual({
      id: "file-pending",
      kind: "conversation_file",
      readable: false,
      ref: "conversation_file:file-pending",
      status: "pending",
      title: pendingSourceTitle,
      visionReady: false,
    })
  })

  it("marks uploaded vision images as selectable source refs", () => {
    expect(
      mapAttachmentToSourceContextSelection({
        fileId: "file-image",
        id: "local-image",
        imageDataUrl: "data:image/png;base64,aW1hZ2U=",
        kind: "image",
        name: imageSourceTitle,
        readable: true,
        status: "uploaded",
      })
    ).toEqual({
      id: "file-image",
      kind: "conversation_file",
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
