import { describe, expect, it } from "vitest"
import {
  createContextRefs,
  mapAttachmentToSourceContextSelection,
  toggleSourceContextSelection,
  type AgentChatSourceContextSelection,
  type AgentChatProductContextSelection,
} from "./agent-chat-context-selection"

const sourceTitle = "brief.md"

describe("agent chat context selection", () => {
  it("combines product refs and explicit source refs without duplicates", () => {
    const products: AgentChatProductContextSelection[] = [
      { id: "product-a", name: "Product A" },
    ]
    const source: AgentChatSourceContextSelection = {
      id: "file-a",
      kind: "conversation_file" as const,
      readable: true,
      ref: "conversation_file:file-a",
      status: "uploaded" as const,
      title: sourceTitle,
    }

    expect(createContextRefs(products, [source, source])).toEqual([
      "product:product-a",
      "conversation_file:file-a",
    ])
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
})
