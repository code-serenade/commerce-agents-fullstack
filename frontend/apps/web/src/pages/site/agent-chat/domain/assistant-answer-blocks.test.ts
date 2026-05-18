import { describe, expect, it } from "vitest"
import { parseAnswerBlocks } from "./assistant-answer-blocks"

describe("assistant answer blocks", () => {
  it("parses markdown tables into table blocks", () => {
    expect(
      parseAnswerBlocks(
        [
          "## Role split",
          "| Role | Finding | Handoff |",
          "| --- | --- | --- |",
          "| Data | Read conversion | Funnel metrics |",
          "| Content | Read selling points | Script |",
        ].join("\n")
      )
    ).toEqual([
      { depth: 2, kind: "heading", text: "Role split" },
      {
        header: ["Role", "Finding", "Handoff"],
        kind: "table",
        rows: [
          ["Data", "Read conversion", "Funnel metrics"],
          ["Content", "Read selling points", "Script"],
        ],
      },
    ])
  })

  it("keeps non-table pipe text as paragraph content", () => {
    expect(parseAnswerBlocks("A | B")).toEqual([
      { kind: "paragraph", text: "A | B" },
    ])
  })
})
