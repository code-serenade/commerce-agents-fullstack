import { describe, expect, it } from "vitest"
import {
  isInternalRuntimeText,
  sanitizeUserFacingProcessText,
} from "./agent-user-facing-runtime-text"

const genericProgress = textFromCodes(24050, 25353, 29992, 25143, 38382, 39064, 25972, 29702, 25104, 21487, 29992, 32467, 35770)
const taskPlanWithRawDifficulty = `${textFromCodes(
  20219,
  21153,
  23433,
  25490
)} complex ${textFromCodes(38590, 24230)}`
const capabilityCheck = `${textFromCodes(
  33021,
  21147,
  26680,
  23545
)} planned 1 requested 1`
const contextReadHint = `${textFromCodes(
  36164,
  26009,
  35835,
  21462,
  25552,
  31034
)} no active workspace context sources were found`
const currentRoleTask = `${textFromCodes(
  24403,
  21069,
  23703,
  20301,
  20219,
  21153
)}: worker.search should use selected_skills`
const t = ((key: string) => {
  if (key === "agentChat.thread.runtimeText.genericProgress") {
    return genericProgress
  }
  return key
}) as never

describe("agent user-facing runtime text", () => {
  it("hides internal adapter and i18n key text from process UI", () => {
    const rawTexts = [
      "adapter_not_implemented",
      "complex difficulty",
      "No enabled platform",
      "no active workspace context sources were found",
      "L3 blocked",
      "agentChat.thread.dispatchFacts.skillAudit.label",
      "failure_kind:worker_unavailable",
      taskPlanWithRawDifficulty,
      capabilityCheck,
      contextReadHint,
      currentRoleTask,
    ]

    rawTexts.forEach((rawText) => {
      expect(isInternalRuntimeText(rawText)).toBe(true)
      expect(sanitizeUserFacingProcessText(rawText, t)).toBe(genericProgress)
    })
  })
})

function textFromCodes(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
