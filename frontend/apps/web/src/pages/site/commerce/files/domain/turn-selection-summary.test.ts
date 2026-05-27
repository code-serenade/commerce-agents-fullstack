import { describe, expect, it } from "vitest"
import type { AgentChatSourceContextSelection } from "../../../agent-chat/domain/agent-chat-context-selection"
import { fileObjectKey } from "./logic"
import { file } from "./test-helpers"
import { summarizeTrainingFilesTurnSelection } from "./turn-selection-summary"

describe("commerce files turn selection summary", () => {
  it("summarizes files selected for the next turn", () => {
    const selected = file({
      fileName: "selected.pdf",
      id: "selected",
      objectKey: "selected.pdf",
    })
    const extra = file({
      fileName: "extra.pdf",
      id: "extra",
      objectKey: "extra.pdf",
    })
    const attached = file({
      fileName: "attached.pdf",
      id: "attached",
      objectKey: "attached.pdf",
    })
    const selectedSources: AgentChatSourceContextSelection[] = [
      selected,
      extra,
    ].map((source) => ({
      bucket: source.bucket,
      id: source.id,
      kind: "conversation_file",
      objectKey: source.objectKey,
      readable: true,
      ref: `conversation_file:${source.id}`,
      status: "uploaded",
      title: source.fileName ?? source.id,
    }))

    expect(
      summarizeTrainingFilesTurnSelection({
        attachedObjectKeys: new Set([
          fileObjectKey(selected),
          fileObjectKey(extra),
          fileObjectKey(attached),
        ]),
        files: [selected, extra, attached],
        maxNames: 1,
        selectedSources,
      })
    ).toEqual({
      hiddenNames: 1,
      selectedNames: ["selected.pdf"],
      total: 2,
    })
  })
})
