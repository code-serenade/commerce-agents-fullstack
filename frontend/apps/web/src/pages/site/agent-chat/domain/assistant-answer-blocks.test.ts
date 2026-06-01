import { describe, expect, it } from "vitest"
import { parseAnswerBlocks } from "./assistant-answer-blocks"
import {
  dateMissing,
  denseBracketComplianceHeading,
  denseBracketComplianceText,
  denseBracketCertainHeading,
  denseBracketCoreHeading,
  denseBracketIntro,
  denseBracketPractice,
  denseBracketPracticeHeading,
  denseBracketPracticeText,
  denseBracketProductFit,
  denseBracketSourceHeading,
  denseBracketSourceText,
  denseColonActionHeading,
  denseColonActionText,
  denseColonFitHeading,
  denseColonFitText,
  denseColonIntro,
  denseColonRiskHeading,
  denseColonRiskText,
  lowPriceSnippet,
  lowPriceTitle,
  ruleCenterSnippet,
  ruleCenterTitle,
  sourceIntro,
  sourceMarker,
} from "./assistant-answer-blocks.fixtures"

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

  it("restores readable blocks from dense bracket-heading answers", () => {
    expect(
      parseAnswerBlocks(
        [
          denseBracketIntro,
          denseBracketCoreHeading,
          denseBracketProductFit,
          denseBracketCertainHeading,
          denseBracketPractice,
          denseBracketSourceHeading,
          denseBracketSourceText,
        ].join("")
      )
    ).toEqual([
      {
        kind: "paragraph",
        text: denseBracketIntro,
      },
      { depth: 3, kind: "heading", text: denseBracketCoreHeading.slice(1, -1) },
      {
        kind: "paragraph",
        text: denseBracketProductFit,
      },
      {
        depth: 3,
        kind: "heading",
        text: denseBracketCertainHeading.slice(1, -1),
      },
      {
        depth: 3,
        kind: "heading",
        text: denseBracketPracticeHeading,
      },
      {
        kind: "paragraph",
        text: denseBracketPracticeText,
      },
      {
        depth: 3,
        kind: "heading",
        text: denseBracketComplianceHeading,
      },
      {
        kind: "paragraph",
        text: denseBracketComplianceText,
      },
      { depth: 3, kind: "heading", text: denseBracketSourceHeading.slice(1, -1) },
      { kind: "paragraph", text: denseBracketSourceText },
    ])
  })

  it("restores readable blocks from dense colon-heading answers", () => {
    expect(
      parseAnswerBlocks(
        [
          denseColonIntro,
          denseColonFitHeading,
          denseColonFitText,
          denseColonRiskHeading,
          denseColonRiskText,
          denseColonActionHeading,
          denseColonActionText,
        ].join("")
      )
    ).toEqual([
      { kind: "paragraph", text: denseColonIntro },
      { depth: 3, kind: "heading", text: denseColonFitHeading.slice(0, -1) },
      { kind: "paragraph", text: denseColonFitText },
      { depth: 3, kind: "heading", text: denseColonRiskHeading.slice(0, -1) },
      { kind: "paragraph", text: denseColonRiskText },
      { depth: 3, kind: "heading", text: denseColonActionHeading.slice(0, -1) },
      { kind: "paragraph", text: denseColonActionText },
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
