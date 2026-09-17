import { describe, expect, it } from "vitest"
import type { AgentChatMessage } from "../types"
import {
  getStudentNextActionGroup,
  getStudentNextActionPrompts,
} from "./student-next-actions-view-model"
import {
  getCopy,
  t,
  textFromCodes,
} from "./student-next-actions-view-model-test-utils"

describe("student next action product routing", () => {
  it("does not switch product questions to report prompts just because the answer mentions report wording", () => {
    const message: AgentChatMessage = {
      id: "assistant-3",
      role: "assistant",
      content: textFromCodes(
        21487,
        20197,
        20808,
        20570,
        20027,
        22270,
        21644,
        26631,
        39064,
        20248,
        21270,
        65292,
        26368,
        21518,
        20877,
        25226,
        32467,
        35770,
        25972,
        29702,
        25104,
        25253,
        21578,
        12290
      ),
    }
    const previousUserContent =
      textFromCodes(
        25105,
        24819,
        21334,
        19968,
        27454,
        22799,
        23395,
        38450,
        26194,
        34915,
        65292,
        20215,
        26684
      ) +
      "129" +
      textFromCodes(
        20803,
        65292,
        38754,
        21521,
        22823,
        23398,
        29983,
        65292,
        20027,
        22270,
        27604,
        36739,
        26222,
        36890,
        65292,
        35831,
        32473,
        19977,
        27493,
        20248,
        21270,
        26041,
        21521,
        12290
      )

    const prompts = getStudentNextActionPrompts(t, message, previousUserContent)

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "productFollowup"
    )
    expect(prompts.join("\n")).toContain(textFromCodes(38450, 26194, 34915))
    expect(prompts.join("\n")).toContain("129")
    expect(prompts).not.toEqual(
      getCopy("agentChat.thread.nextActions.groups.report")
    )
  })

  it("keeps competitor and small-test product questions out of generic templates", () => {
    const message: AgentChatMessage = {
      id: "assistant-2",
      role: "assistant",
      content: textFromCodes(
        21487,
        20197,
        20808,
        23545,
        27604,
        31454,
        21697,
        65292,
        20877,
        20570,
        23567,
        27979,
        39564,
        35777,
        12290
      ),
    }
    const previousUserContent = textFromCodes(
      36825,
      27454,
      27700,
      26479,
      21487,
      20197,
      24590,
      20040,
      21644,
      31454,
      21697,
      23545,
      27604,
      65292,
      22914,
      26524,
      20570,
      23567,
      27979,
      35201,
      30475,
      28857,
      20987,
      21644,
      36716,
      21270,
      21527,
      65311
    )

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "productFollowup"
    )
    expect(
      getStudentNextActionPrompts(t, message, previousUserContent).join("\n")
    ).toContain(textFromCodes(22260, 32469))
  })
})
