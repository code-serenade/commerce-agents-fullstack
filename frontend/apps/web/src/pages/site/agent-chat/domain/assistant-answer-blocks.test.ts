import { describe, expect, it } from "vitest"
import { parseAnswerBlocks } from "./assistant-answer-blocks"

const sourceIntro = String.fromCharCode(
  0x672c,
  0x8f6e,
  0x5df2,
  0x901a,
  0x8fc7,
  0x5916,
  0x90e8,
  0x641c,
  0x7d22,
  0x53d6,
  0x5f97,
  0x5b9e,
  0x65f6,
  0x6765,
  0x6e90,
  0xff1b,
  0x4ee5,
  0x4e0b,
  0x4f9d,
  0x636e,
  0x5217,
  0x8868,
  0x4e3a,
  0x540e,
  0x7eed,
  0x5224,
  0x65ad,
  0x7684,
  0x51fa,
  0x5904,
  0xff0c,
  0x82e5,
  0x6765,
  0x6e90,
  0x4e0e,
  0x901a,
  0x7528,
  0x7ecf,
  0x9a8c,
  0x51b2,
  0x7a81,
  0xff0c,
  0x4ee5,
  0x5b9e,
  0x65f6,
  0x6765,
  0x6e90,
  0x4e3a,
  0x51c6,
  0x3002
)
const sourceMarker = String.fromCharCode(
  0x5b9e,
  0x65f6,
  0x641c,
  0x7d22,
  0x6765,
  0x6e90,
  0xff08,
  0x73,
  0x65,
  0x61,
  0x72,
  0x63,
  0x68,
  0x5f,
  0x70,
  0x6c,
  0x61,
  0x74,
  0x66,
  0x6f,
  0x72,
  0x6d,
  0x5f,
  0x70,
  0x6f,
  0x6c,
  0x69,
  0x63,
  0x79,
  0xff09,
  0xff1a,
  0x32,
  0x20,
  0x4e2a,
  0x53ef,
  0x91c7,
  0x4fe1,
  0x6765,
  0x6e90
)
const ruleCenterTitle = String.fromCharCode(
  0x6296,
  0x97f3,
  0x7535,
  0x5546,
  0x5b66,
  0x4e60,
  0x4e2d,
  0x5fc3,
  0x2d,
  0x89c4,
  0x5219,
  0x4e2d,
  0x5fc3
)
const ruleCenterSnippet = String.fromCharCode(
  0x6296,
  0x97f3,
  0x7535,
  0x5546,
  0x5b98,
  0x65b9,
  0x89c4,
  0x5219,
  0x4e2d,
  0x5fc3,
  0x3002
)
const lowPriceTitle = String.fromCharCode(
  0x4f4e,
  0x4ef7,
  0x5f15,
  0x6d41,
  0x4e13,
  0x9879,
  0x6cbb,
  0x7406,
  0x516c,
  0x544a,
  0x2d,
  0x6296,
  0x97f3,
  0x7535,
  0x5546,
  0x5b66,
  0x4e60,
  0x4e2d,
  0x5fc3
)
const dateMissing = String.fromCharCode(
  0x53d1,
  0x5e03,
  0x65e5,
  0x671f,
  0x672a,
  0x6807,
  0x6ce8
)
const lowPriceSnippet = String.fromCharCode(
  0x53ef,
  0x4f5c,
  0x4e3a,
  0x4f4e,
  0x4ef7,
  0x5f15,
  0x6d41,
  0x6cbb,
  0x7406,
  0x89c4,
  0x5219,
  0x6765,
  0x6e90,
  0x3002
)

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

  it("parses inline external search sources into source cards", () => {
    expect(
      parseAnswerBlocks(
        [
          sourceIntro,
          sourceMarker,
          `${ruleCenterTitle} | 2026-05-20 | https://school.jinritemai.com/doudian/web/rules | ${ruleCenterSnippet}`,
          `${lowPriceTitle} | ${dateMissing} | https://school.jinritemai.com/doudian/wap/article/aHTNBDPM7VUG | ${lowPriceSnippet}`,
        ].join(" ")
      )
    ).toEqual([
      {
        kind: "paragraph",
        text: sourceIntro,
      },
      {
        intro: sourceMarker,
        kind: "sources",
        sources: [
          {
            publishedAt: "2026-05-20",
            snippet: ruleCenterSnippet,
            title: ruleCenterTitle,
            url: "https://school.jinritemai.com/doudian/web/rules",
          },
          {
            publishedAt: dateMissing,
            snippet: lowPriceSnippet,
            title: lowPriceTitle,
            url: "https://school.jinritemai.com/doudian/wap/article/aHTNBDPM7VUG",
          },
        ],
      },
    ])
  })
})
