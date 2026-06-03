import { describe, expect, it } from "vitest"
import { parseAnswerBlocks } from "./assistant-answer-blocks"
import {
  plainCoffeeChannelHeading,
  plainCoffeeDetailHeading,
  plainCoffeeDetailText,
  plainCoffeeFirstStepHeading,
  plainCoffeeFirstStepText,
  plainCoffeeIntro,
  plainCoffeeMaterialHeading,
  plainCoffeeMoreInfoHeading,
  plainCoffeeObjectHeading,
  plainCoffeeObjectText,
  plainCoffeeQuestion,
  plainCoffeeRiskHeading,
  plainCoffeeRiskText,
  plainCoffeeTitleHeading,
  tabTableHeaderCategory,
  tabTableHeaderFields,
  tabTableHeading,
  tabTableHeadingText,
  tabTableProductBase,
  tabTableProductFields,
  tabTableTraffic,
  tabTableTrafficFields,
} from "./assistant-answer-blocks.fixtures"

describe("assistant answer fallback formatting", () => {
  it("parses tab-separated table answers when markdown separators are missing", () => {
    expect(
      parseAnswerBlocks(
        [
          tabTableHeading,
          `${tabTableHeaderCategory}\t${tabTableHeaderFields}`,
          `${tabTableProductBase}\t${tabTableProductFields}`,
          `${tabTableTraffic}\t${tabTableTrafficFields}`,
        ].join("\n")
      )
    ).toEqual([
      { depth: 3, kind: "heading", text: tabTableHeadingText },
      {
        header: [tabTableHeaderCategory, tabTableHeaderFields],
        kind: "table",
        rows: [
          [tabTableProductBase, tabTableProductFields],
          [tabTableTraffic, tabTableTrafficFields],
        ],
      },
    ])
  })

  it("keeps plain newline answers readable when the model omits markdown markers", () => {
    expect(
      parseAnswerBlocks(
        [
          plainCoffeeIntro,
          plainCoffeeObjectHeading,
          plainCoffeeObjectText,
          plainCoffeeChannelHeading,
          `${plainCoffeeFirstStepHeading}：${plainCoffeeFirstStepText}`,
          plainCoffeeMaterialHeading,
          plainCoffeeTitleHeading,
          `${plainCoffeeDetailHeading}：${plainCoffeeDetailText}`,
          plainCoffeeRiskHeading,
          plainCoffeeRiskText,
          plainCoffeeMoreInfoHeading,
          plainCoffeeQuestion,
        ].join("\n")
      )
    ).toEqual([
      {
        kind: "paragraph",
        text: plainCoffeeIntro,
      },
      { depth: 3, kind: "heading", text: plainCoffeeObjectHeading },
      {
        kind: "paragraph",
        text: plainCoffeeObjectText,
      },
      { depth: 3, kind: "heading", text: plainCoffeeChannelHeading },
      { depth: 3, kind: "heading", text: plainCoffeeFirstStepHeading },
      {
        kind: "paragraph",
        text: plainCoffeeFirstStepText,
      },
      { depth: 3, kind: "heading", text: plainCoffeeMaterialHeading },
      { depth: 3, kind: "heading", text: plainCoffeeTitleHeading },
      { depth: 3, kind: "heading", text: plainCoffeeDetailHeading },
      {
        kind: "paragraph",
        text: plainCoffeeDetailText,
      },
      { depth: 3, kind: "heading", text: plainCoffeeRiskHeading },
      {
        kind: "paragraph",
        text: plainCoffeeRiskText,
      },
      { depth: 3, kind: "heading", text: plainCoffeeMoreInfoHeading },
      {
        kind: "paragraph",
        text: plainCoffeeQuestion,
      },
    ])
  })
})
