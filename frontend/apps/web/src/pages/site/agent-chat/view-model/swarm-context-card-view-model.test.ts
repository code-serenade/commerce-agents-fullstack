import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getContextCard } from "./swarm-context-card-view-model"

const packTitle = "Workspace pack"
const packMeta = "1/3 readable"
const packReadyDescription = "ready"
const itemLine = "spreadsheet SKU sheet"
const warningLine = "warning unsupported pdf"
const omittedLine = "omitted file-b"
const itemTitle = ["SKU", "sheet"].join(" ")

const t = ((key: string, params?: Record<string, unknown>) => {
  if (key === "agentChat.swarm.cards.workspaceContext.packTitle") return packTitle
  if (key === "agentChat.swarm.cards.workspaceContext.packMeta") {
    return `${params?.readable}/${params?.total} readable`
  }
  if (key === "agentChat.swarm.cards.workspaceContext.packReadyDescription") {
    return packReadyDescription
  }
  if (key === "agentChat.swarm.cards.workspaceContext.packPartialDescription") {
    return "partial"
  }
  if (key === "agentChat.swarm.cards.workspaceContext.packItemLine") {
    return `${params?.kind} ${params?.title}`
  }
  if (key === "agentChat.swarm.cards.workspaceContext.warningLine") {
    return `warning ${params?.warning}`
  }
  if (key === "agentChat.swarm.cards.workspaceContext.omittedLine") {
    return `omitted ${params?.ref}`
  }
  return key
}) as never

describe("getContextCard", () => {
  it("shows omitted material refs from workspace context pack events", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "workspace_context_pack_loaded",
        items: [
          {
            content_kind: "spreadsheet",
            id: "file-a",
            text: "sku data",
            title: itemTitle,
          },
          { id: "file-b" },
          { id: "file-c" },
        ],
        omitted: ["file-b", "file-c"],
        readiness: "partial",
        warnings: ["unsupported pdf"],
      },
    ]

    expect(getContextCard(events, t)).toMatchObject({
      description: packReadyDescription,
      lines: [itemLine, warningLine, "omitted file-b", "omitted file-c"],
      meta: packMeta,
      title: packTitle,
    })
  })

  it("keeps omitted refs visible when the material list is short", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "workspace_context_pack_loaded",
        items: [
          {
            content_kind: "spreadsheet",
            id: "file-a",
            text: "sku data",
            title: itemTitle,
          },
        ],
        omitted: ["file-b"],
        readiness: "partial",
        warnings: ["unsupported pdf"],
      },
    ]

    expect(getContextCard(events, t)?.lines).toEqual([itemLine, warningLine, omittedLine])
  })
})
