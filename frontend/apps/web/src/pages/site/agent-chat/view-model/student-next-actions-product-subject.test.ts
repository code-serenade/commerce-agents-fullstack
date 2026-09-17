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

describe("student next action product subject", () => {
  it("uses the original storage box subject instead of later answer wording", () => {
    const storageBox = textFromCodes(
      23398,
      29983,
      23487,
      33293,
      29992,
      26700,
      38754,
      25910,
      32435,
      30418
    )
    const previousUserContent =
      textFromCodes(25105, 24819, 21334, 19968, 27454) +
      storageBox +
      textFromCodes(65292, 21806, 20215) +
      "39" +
      textFromCodes(
        20803,
        65292,
        21482,
        26377,
        26222,
        36890,
        30333,
        24213,
        22270,
        65292,
        27809,
        26377,
        38144,
        37327,
        25968,
        25454,
        12290
      )
    const message: AgentChatMessage = {
      id: "assistant-storage-box",
      role: "assistant",
      content: textFromCodes(
        19979,
        19968,
        27493,
        26368,
        20540,
        24471,
        20808,
        20570,
        30340,
        26159,
        37325,
        20570,
        19968,
        29256,
        20027,
        22270,
        21644,
        26631,
        39064
      ),
    }

    const prompts = getStudentNextActionPrompts(
      t,
      message,
      previousUserContent
    ).join("\n")

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "productFollowup"
    )
    expect(prompts).toContain(storageBox)
    expect(prompts).toContain("39")
    expect(prompts).not.toContain(
      textFromCodes(30340, 26159, 37325, 20570, 19968, 29256)
    )
  })

  it("keeps official source lookup answers out of product follow-up prompts", () => {
    const previousUserContent =
      textFromCodes(
        35831,
        32852,
        32593,
        25628,
        32034,
        24182,
        32473,
        20986,
        26469,
        28304,
        38142,
        25509,
        65306,
        26368,
        36817
      ) +
      "30" +
      textFromCodes(
        22825,
        25238,
        38899,
        30005,
        21830,
        20851,
        20110,
        20215,
        26684,
        21512,
        35268,
        12289,
        20302,
        20215,
        24341,
        27969,
        12289,
        34394,
        20551,
        23459,
        20256,
        30340,
        23448,
        26041,
        20844,
        24320,
        35268,
        21017,
        25110,
        20844,
        21578,
        65307,
        21482,
        36755,
        20986,
        26469,
        28304,
        26631,
        39064,
        12289,
        21457,
        24067,
        26085,
        26399,
        21644,
        36866,
        29992,
        39118,
        38505,
        28857
      )
    const message: AgentChatMessage = {
      id: "assistant-source-lookup",
      role: "assistant",
      content: textFromCodes(
        26412,
        36718,
        26410,
        25343,
        21040,
        23454,
        26102,
        26469,
        28304,
        65292,
        19981,
        33021,
        24403,
        20316,
        26368,
        26032,
        35268,
        21017,
        32467,
        35770
      ),
    }

    const prompts = getStudentNextActionPrompts(t, message, previousUserContent)

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "default"
    )
    expect(prompts).toEqual(
      getCopy("agentChat.thread.nextActions.groups.default")
    )
  })
})
