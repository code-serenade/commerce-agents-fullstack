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
  const fallbackStatusHeading = textFromCodes([24403, 21069, 29366, 24577])
  const fallbackStatusParagraph = textFromCodes([
    27169, 22411, 26381, 21153, 20020, 26102, 19981, 21487, 29992, 65292,
    19978, 28216, 27169, 22411, 26381, 21153, 36820, 22238, 26381, 21153,
    31471, 38169, 35823, 65292, 25152, 20197, 26412, 36718, 27809, 26377,
    24418, 25104, 21487, 38752, 22238, 31572, 12290,
  ])
  const fallbackImpactHeading = textFromCodes([
    23545, 26412, 36718, 22238, 31572, 30340, 24433, 21709,
  ])
  const fallbackReceivedItem = textFromCodes([
    24050, 25910, 21040, 20320, 30340, 38382, 39064, 21644, 26412, 36718,
    36873, 25321, 30340, 36164, 26009, 12290,
  ])
  const fallbackGenerationItem = textFromCodes([
    38656, 35201, 27169, 22411, 29983, 25104, 30340, 20998, 26512, 12289,
    24635, 32467, 21644, 21327, 20316, 32467, 26524, 26242, 26102, 26080,
    27861, 23436, 25104, 12290,
  ])
  const fallbackNotJudgementItem = textFromCodes([
    36825, 19981, 26159, 21830, 21697, 12289, 27963, 21160, 25110, 36164,
    26009, 26412, 36523, 30340, 21028, 26029, 32467, 26524, 12290,
  ])
  const fallbackNextStepHeading = textFromCodes([19979, 19968, 27493])
  const fallbackRetryItem = textFromCodes([
    21487, 20197, 31245, 21518, 37325, 35797, 21516, 19968, 20010, 38382,
    39064, 12290,
  ])
  const fallbackAdminItem = textFromCodes([
    22914, 26524, 36830, 32493, 20986, 29616, 65292, 35831, 32852, 31995,
    31649, 29702, 21592, 26816, 26597, 27169, 22411, 26381, 21153, 29366,
    24577, 12290,
  ])

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

  it("parses structured model-failure fallbacks into readable blocks", () => {
    expect(
      parseAnswerBlocks(
        [
          `## ${fallbackStatusHeading}`,
          "",
          fallbackStatusParagraph,
          "",
          `## ${fallbackImpactHeading}`,
          "",
          `- ${fallbackReceivedItem}`,
          `- ${fallbackGenerationItem}`,
          `- ${fallbackNotJudgementItem}`,
          "",
          `## ${fallbackNextStepHeading}`,
          "",
          `- ${fallbackRetryItem}`,
          `- ${fallbackAdminItem}`,
        ].join("\n")
      )
    ).toEqual([
      { depth: 2, kind: "heading", text: fallbackStatusHeading },
      {
        kind: "paragraph",
        text: fallbackStatusParagraph,
      },
      { depth: 2, kind: "heading", text: fallbackImpactHeading },
      {
        items: [
          fallbackReceivedItem,
          fallbackGenerationItem,
          fallbackNotJudgementItem,
        ],
        kind: "list",
        ordered: false,
      },
      { depth: 2, kind: "heading", text: fallbackNextStepHeading },
      {
        items: [fallbackRetryItem, fallbackAdminItem],
        kind: "list",
        ordered: false,
      },
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

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}
