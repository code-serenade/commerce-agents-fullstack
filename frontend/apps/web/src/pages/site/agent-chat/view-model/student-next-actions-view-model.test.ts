import { describe, expect, it } from "vitest"
import type { AgentChatMessage } from "../types"
import {
  getStudentNextActionGroup,
  getStudentNextActionMeta,
  getStudentNextActionPrompts,
} from "./student-next-actions-view-model"
import { t, textFromCodes } from "./student-next-actions-view-model-test-utils"

describe("student next action view model", () => {
  it("keeps product follow-up prompts contextual even when the answer mentions reports", () => {
    const message: AgentChatMessage = {
      id: "assistant-1",
      role: "assistant",
      content: textFromCodes(
        36825,
        36718,
        20808,
        21028,
        26029,
        33021,
        19981,
        33021,
        20570,
        65292,
        21518,
        32493,
        20063,
        21487,
        20197,
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
        12290,
        35831,
        20687,
        24179,
        31561,
        20249,
        20276,
        19968,
        26679,
        20808,
        21028,
        26029,
        33021,
        19981,
        33021,
        20570,
        12290
      )

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "productFollowup"
    )
    expect(getStudentNextActionMeta(t, message, previousUserContent)).toEqual({
      detail: textFromCodes(
        22260,
        32469,
        36825,
        20214,
        21830,
        21697,
        32487,
        32493,
        36861,
        38382
      ),
      group: "productFollowup",
      label: textFromCodes(21830, 21697, 36861, 38382),
    })
    expect(
      getStudentNextActionPrompts(t, message, previousUserContent)
    ).toEqual([
      textFromCodes(
        22260,
        32469,
        22799,
        23395,
        38450,
        26194,
        34915,
        65292,
        20808,
        25226,
        20027,
        22270,
        34920,
        36798,
        26368,
        35813,
        25913,
        30340
      ) +
        " 3 " +
        textFromCodes(20010, 28857, 21015, 20986, 26469),
      textFromCodes(
        32473,
        25105,
        19968,
        29256,
        36866,
        21512,
        22823,
        23398,
        29983,
        30340,
        22799,
        23395,
        38450,
        26194,
        34915,
        33258,
        28982,
        26631,
        39064
      ),
      textFromCodes(25353) +
        "129" +
        textFromCodes(
          20803,
          12289,
          21334,
          28857,
          21644,
          22823,
          23398,
          29983,
          21028,
          26029,
          22799,
          23395,
          38450,
          26194,
          34915,
          20540,
          19981,
          20540,
          24471,
          23567,
          27979
        ),
      textFromCodes(
        19979,
        19968,
        36718,
        39564,
        35777,
        22799,
        23395,
        38450,
        26194,
        34915,
        26102,
        65292,
        24212,
        35813,
        35760,
        24405,
        21738,
        20123,
        25968,
        25454
      ),
    ])
  })
})
