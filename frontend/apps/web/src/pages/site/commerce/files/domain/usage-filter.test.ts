import { describe, expect, it } from "vitest"
import type { AgentChatSourceContextSelection } from "../../../agent-chat/domain/agent-chat-context-selection"
import { file } from "./test-helpers"
import { fileObjectKey } from "./logic"
import { filterTrainingFilesByUsage } from "./usage-filter"

describe("commerce files usage filter", () => {
  it("filters files by current conversation and turn usage state", () => {
    const available = file({ id: "available", objectKey: "available.pdf" })
    const attached = file({ id: "attached", objectKey: "attached.pdf" })
    const selected = file({ id: "selected", objectKey: "selected.pdf" })
    const selectedSources: AgentChatSourceContextSelection[] = [
      {
        bucket: selected.bucket,
        id: selected.id,
        kind: "conversation_file",
        objectKey: selected.objectKey,
        readable: true,
        ref: `conversation_file:${selected.id}`,
        status: "uploaded",
        title: ["selected", "pdf"].join("."),
      },
    ]

    const input = {
      attachedObjectKeys: new Set([fileObjectKey(attached)]),
      files: [available, attached, selected],
      selectedSources,
    }

    expect(
      filterTrainingFilesByUsage({ ...input, usage: "all" }).map(
        (item) => item.id
      )
    ).toEqual(["available", "attached", "selected"])
    expect(
      filterTrainingFilesByUsage({ ...input, usage: "available" }).map(
        (item) => item.id
      )
    ).toEqual(["available"])
    expect(
      filterTrainingFilesByUsage({ ...input, usage: "attached" }).map(
        (item) => item.id
      )
    ).toEqual(["attached"])
    expect(
      filterTrainingFilesByUsage({ ...input, usage: "selectedForTurn" }).map(
        (item) => item.id
      )
    ).toEqual(["selected"])
  })
})
