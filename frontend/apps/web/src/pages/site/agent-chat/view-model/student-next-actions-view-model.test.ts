import { describe, expect, it } from "vitest"
import type { AgentChatMessage } from "../types"
import {
  getStudentNextActionGroup,
  getStudentNextActionMeta,
  getStudentNextActionPrompts,
} from "./student-next-actions-view-model"

const copy: Record<string, unknown> = {
  "agentChat.thread.nextActions.contextFallback.audience": textFromCodes(24403, 21069, 20154, 32676),
  "agentChat.thread.nextActions.contextFallback.issue": textFromCodes(24403, 21069, 38382, 39064),
  "agentChat.thread.nextActions.contextFallback.price": textFromCodes(24403, 21069, 20215, 26684),
  "agentChat.thread.nextActions.contextFallback.subject": textFromCodes(24403, 21069, 21830, 21697),
  "agentChat.thread.nextActions.contextIssue.mainImage": textFromCodes(20027, 22270, 34920, 36798),
  "agentChat.thread.nextActions.contextualProductFollowup.mainImage":
    `${textFromCodes(22260, 32469)}{{subject}}${textFromCodes(65292, 20808, 25226)}{{issue}}${textFromCodes(26368, 35813, 25913, 30340)} 3 ${textFromCodes(20010, 28857, 21015, 20986, 26469)}`,
  "agentChat.thread.nextActions.contextualProductFollowup.title":
    `${textFromCodes(32473, 25105, 19968, 29256, 36866, 21512)}{{audience}}${textFromCodes(30340)}{{subject}}${textFromCodes(33258, 28982, 26631, 39064)}`,
  "agentChat.thread.nextActions.contextualProductFollowup.validation":
    `${textFromCodes(19979, 19968, 36718, 39564, 35777)}{{subject}}${textFromCodes(26102, 65292, 24212, 35813, 35760, 24405, 21738, 20123, 25968, 25454)}`,
  "agentChat.thread.nextActions.contextualProductFollowup.value":
    `${textFromCodes(25353)}{{price}}${textFromCodes(12289, 21334, 28857, 21644)}{{audience}}${textFromCodes(21028, 26029)}{{subject}}${textFromCodes(20540, 19981, 20540, 24471, 23567, 27979)}`,
  "agentChat.thread.nextActions.groups.default": [
    textFromCodes(32487, 32493, 36861, 38382, 19968, 20010, 26356, 20855, 20307, 30340, 30005, 21830, 38382, 39064),
  ],
  "agentChat.thread.nextActions.groups.report": [
    textFromCodes(24110, 25105, 25972, 29702, 25104, 25253, 21578),
  ],
  "agentChat.thread.nextActions.meta.productFollowup.detail":
    textFromCodes(22260, 32469, 36825, 20214, 21830, 21697, 32487, 32493, 36861, 38382),
  "agentChat.thread.nextActions.meta.productFollowup.label": textFromCodes(21830, 21697, 36861, 38382),
}

const t = ((key: string, params?: Record<string, unknown>) => {
  const value = copy[key]
  if (typeof value === "string") {
    return Object.entries(params ?? {}).reduce(
      (current, [name, replacement]) =>
        current.replaceAll(`{{${name}}}`, String(replacement)),
      value
    )
  }
  return value ?? key
}) as never

describe("student next action view model", () => {
  it("keeps product follow-up prompts contextual even when the answer mentions reports", () => {
    const message: AgentChatMessage = {
      id: "assistant-1",
      role: "assistant",
      content: textFromCodes(36825, 36718, 20808, 21028, 26029, 33021, 19981, 33021, 20570, 65292, 21518, 32493, 20063, 21487, 20197, 25972, 29702, 25104, 25253, 21578, 12290),
    }
    const previousUserContent =
      textFromCodes(25105, 24819, 21334, 19968, 27454, 22799, 23395, 38450, 26194, 34915, 65292, 20215, 26684) +
      "129" +
      textFromCodes(20803, 65292, 38754, 21521, 22823, 23398, 29983, 65292, 20027, 22270, 27604, 36739, 26222, 36890, 12290, 35831, 20687, 24179, 31561, 20249, 20276, 19968, 26679, 20808, 21028, 26029, 33021, 19981, 33021, 20570, 12290)

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "productFollowup"
    )
    expect(getStudentNextActionMeta(t, message, previousUserContent)).toEqual({
      detail: textFromCodes(22260, 32469, 36825, 20214, 21830, 21697, 32487, 32493, 36861, 38382),
      group: "productFollowup",
      label: textFromCodes(21830, 21697, 36861, 38382),
    })
    expect(
      getStudentNextActionPrompts(t, message, previousUserContent)
    ).toEqual([
      textFromCodes(22260, 32469, 22799, 23395, 38450, 26194, 34915, 65292, 20808, 25226, 20027, 22270, 34920, 36798, 26368, 35813, 25913, 30340) +
        " 3 " +
        textFromCodes(20010, 28857, 21015, 20986, 26469),
      textFromCodes(32473, 25105, 19968, 29256, 36866, 21512, 22823, 23398, 29983, 30340, 22799, 23395, 38450, 26194, 34915, 33258, 28982, 26631, 39064),
      textFromCodes(25353) +
        "129" +
        textFromCodes(20803, 12289, 21334, 28857, 21644, 22823, 23398, 29983, 21028, 26029, 22799, 23395, 38450, 26194, 34915, 20540, 19981, 20540, 24471, 23567, 27979),
      textFromCodes(19979, 19968, 36718, 39564, 35777, 22799, 23395, 38450, 26194, 34915, 26102, 65292, 24212, 35813, 35760, 24405, 21738, 20123, 25968, 25454),
    ])
  })

  it("does not switch product questions to report prompts just because the answer mentions report wording", () => {
    const message: AgentChatMessage = {
      id: "assistant-3",
      role: "assistant",
      content:
        textFromCodes(21487, 20197, 20808, 20570, 20027, 22270, 21644, 26631, 39064, 20248, 21270, 65292, 26368, 21518, 20877, 25226, 32467, 35770, 25972, 29702, 25104, 25253, 21578, 12290),
    }
    const previousUserContent =
      textFromCodes(25105, 24819, 21334, 19968, 27454, 22799, 23395, 38450, 26194, 34915, 65292, 20215, 26684) +
      "129" +
      textFromCodes(20803, 65292, 38754, 21521, 22823, 23398, 29983, 65292, 20027, 22270, 27604, 36739, 26222, 36890, 65292, 35831, 32473, 19977, 27493, 20248, 21270, 26041, 21521, 12290)

    const prompts = getStudentNextActionPrompts(t, message, previousUserContent)

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "productFollowup"
    )
    expect(prompts.join("\n")).toContain(textFromCodes(38450, 26194, 34915))
    expect(prompts.join("\n")).toContain("129")
    expect(prompts).not.toEqual(copy["agentChat.thread.nextActions.groups.report"])
  })

  it("keeps competitor and small-test product questions out of generic templates", () => {
    const message: AgentChatMessage = {
      id: "assistant-2",
      role: "assistant",
      content: textFromCodes(21487, 20197, 20808, 23545, 27604, 31454, 21697, 65292, 20877, 20570, 23567, 27979, 39564, 35777, 12290),
    }
    const previousUserContent =
      textFromCodes(36825, 27454, 27700, 26479, 21487, 20197, 24590, 20040, 21644, 31454, 21697, 23545, 27604, 65292, 22914, 26524, 20570, 23567, 27979, 35201, 30475, 28857, 20987, 21644, 36716, 21270, 21527, 65311)

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "productFollowup"
    )
    expect(
      getStudentNextActionPrompts(t, message, previousUserContent).join("\n")
    ).toContain(textFromCodes(22260, 32469))
  })

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

    const prompts = getStudentNextActionPrompts(
      t,
      message,
      previousUserContent
    )

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "default"
    )
    expect(prompts).toEqual(copy["agentChat.thread.nextActions.groups.default"])
  })
})

function textFromCodes(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
