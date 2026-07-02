import { describe, expect, it } from "vitest"
import zhMessages from "@/../lang/zhCN/messages"

describe("agent chat process copy", () => {
  it("uses task process wording instead of abstract evidence wording", () => {
    const agentChatCopy = JSON.stringify(zhMessages.agentChat)

    expect(zhMessages.agentChat.workbench.title).toBe(
      textFromCodes(20219, 21153, 36807, 31243)
    )
    expect(zhMessages.agentChat.thread.openWorkbench).toBe(
      textFromCodes(26597, 30475, 20219, 21153, 36807, 31243)
    )
    expect(zhMessages.agentChat.thread.executionTitle).toBe(
      textFromCodes(26412, 36718, 20219, 21153, 36807, 31243)
    )
    expect(zhMessages.agentChat.thread.dispatchPanel.title).toBe(
      textFromCodes(26412, 36718, 20219, 21153, 36807, 31243)
    )
    expect(zhMessages.agentChat.thread.artifactsTitle).toBe(
      textFromCodes(36807, 31243, 26448, 26009)
    )
    expect(zhMessages.agentChat.thread.artifacts.waitingBackend).toBe(
      textFromCodes(31561, 24453, 29983, 25104, 26412, 36718, 36807, 31243, 26448, 26009, 12290)
    )
    expect(zhMessages.agentChat.thread.nextActions.meta.report.label).toBe(
      textFromCodes(22797, 30424, 25253, 21578, 24314, 35758)
    )
    expect(agentChatCopy).not.toContain(
      textFromCodes(26597, 30475, 21028, 26029, 20381, 25454)
    )
    expect(agentChatCopy).not.toContain(
      textFromCodes(23637, 24320, 21028, 26029, 20381, 25454)
    )
    expect(agentChatCopy).not.toContain(
      textFromCodes(25253, 21578, 35821, 22659)
    )
    expect(agentChatCopy).not.toContain(
      textFromCodes(21028, 26029, 33033, 32476)
    )
    ;[
      [30452, 25509, 35828, 38382, 39064, 65292, 25105, 20808, 32473, 21028, 26029],
      [24555, 36895, 21028, 26029],
      [27491, 22312, 25972, 29702, 26412, 36718, 21028, 26029],
      [21028, 26029, 24050, 27719, 24635],
      [25972, 29702, 24517, 35201, 20381, 25454],
      [22788, 29702, 21028, 26029],
      [26412, 36718, 21028, 26029, 35828, 26126],
      [27493, 39588, 12289, 20381, 25454],
      [39118, 38505, 12289, 20381, 25454],
      [34917, 20805, 25253, 21578, 37324, 30340, 36807, 31243, 12289, 20381, 25454],
      [21028, 26029, 20381, 25454],
    ].forEach((codes) => {
      expect(agentChatCopy).not.toContain(textFromCodes(...codes))
    })
    expect(agentChatCopy).toContain(
      textFromCodes(32467, 35770, 24050, 27719, 24635)
    )
    expect(agentChatCopy).toContain(
      textFromCodes(25972, 29702, 24517, 35201, 21442, 32771, 26448, 26009)
    )
    expect(agentChatCopy).toContain(
      textFromCodes(22788, 29702, 32467, 35770)
    )
    expect(agentChatCopy).toContain(
      textFromCodes(26412, 36718, 32467, 35770, 35828, 26126)
    )
  })
})

function textFromCodes(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
