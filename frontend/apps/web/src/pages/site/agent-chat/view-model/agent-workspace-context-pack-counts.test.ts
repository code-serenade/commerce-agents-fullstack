import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import {
  getWorkspaceContextPackItemCount,
  getWorkspaceContextPackReadableCount,
  getWorkspaceContextPackUnreadableCount,
  hasWorkspaceContextPackItemRefs,
} from "./agent-workspace-context-pack-counts"

describe("workspace context pack counts", () => {
  it("prefers safe backend counts over raw item bodies", () => {
    const event = {
      type: "workspace_context_pack_loaded",
      itemCount: 6,
      items: [
        {
          id: "file-a",
          text: "hidden raw text",
        },
      ],
      readableItemCount: 4,
    } as Extract<AgentRuntimeEvent, { type: "workspace_context_pack_loaded" }>

    expect(getWorkspaceContextPackItemCount(event)).toBe(6)
    expect(getWorkspaceContextPackReadableCount(event)).toBe(4)
    expect(getWorkspaceContextPackUnreadableCount(event)).toBe(2)
  })

  it("falls back to legacy item text metadata while old events still exist", () => {
    const event = {
      type: "workspace_context_pack_loaded",
      items: [
        {
          id: "file-a",
          textChars: 20,
        },
        {
          id: "file-b",
          text: "legacy text",
        },
        {
          id: "file-c",
        },
      ],
    } as Extract<AgentRuntimeEvent, { type: "workspace_context_pack_loaded" }>

    expect(getWorkspaceContextPackItemCount(event)).toBe(3)
    expect(getWorkspaceContextPackReadableCount(event)).toBe(2)
    expect(getWorkspaceContextPackUnreadableCount(event)).toBe(1)
    expect(hasWorkspaceContextPackItemRefs(event)).toBe(true)
  })

  it("allows safe-count-only events without item refs", () => {
    const event = {
      type: "workspace_context_pack_loaded",
      item_count: 2,
      readable_item_count: 1,
    } as Extract<AgentRuntimeEvent, { type: "workspace_context_pack_loaded" }>

    expect(getWorkspaceContextPackItemCount(event)).toBe(2)
    expect(getWorkspaceContextPackReadableCount(event)).toBe(1)
    expect(getWorkspaceContextPackUnreadableCount(event)).toBe(1)
    expect(hasWorkspaceContextPackItemRefs(event)).toBe(false)
  })
})
