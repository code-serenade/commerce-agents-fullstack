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

  it("maps only uploaded conversation files into selectable source refs", () => {
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
    })

    expect(
      mapAttachmentToSourceContextSelection({
        id: "local-b",
        name: "pending.pdf",
        readable: false,
        status: "uploading",
      })
    ).toBeNull()
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
