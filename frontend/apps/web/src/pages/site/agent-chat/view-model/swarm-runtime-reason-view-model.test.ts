import { describe, expect, it } from "vitest"
import {
  formatUserFacingRuntimeReason,
  isInternalRuntimeReason,
} from "./swarm-runtime-reason-view-model"

const modelUnavailable = textFromCodes([
  27169, 22411, 26381, 21153, 20020, 26102, 19981, 21487, 29992, 65292,
  24403, 21069, 27493, 39588, 27809, 26377, 24418, 25104, 21487, 38752,
  32467, 26524, 12290,
])
const modelConfigUnavailable = textFromCodes([
  27169, 22411, 26381, 21153, 37197, 32622, 19981, 21487, 29992, 65292,
  24403, 21069, 27493, 39588, 27809, 26377, 24418, 25104, 21487, 38752,
  32467, 26524, 12290,
])
const workerUnavailable = textFromCodes([
  21327, 20316, 27493, 39588, 27809, 26377, 25104, 21151, 21551, 21160,
  12290,
])
const timedOut = textFromCodes([
  21327, 20316, 27493, 39588, 25191, 34892, 36229, 26102, 12290,
])
const collaborationIncomplete = textFromCodes([
  21327, 20316, 27493, 39588, 27809, 26377, 24418, 25104, 21487, 29992,
  32467, 26524, 12290,
])
const modelUnavailableShort = textFromCodes([
  27169, 22411, 26381, 21153, 20020, 26102, 19981, 21487, 29992,
])

const t = ((key: string) => {
  const messages: Record<string, string> = {
    "agentChat.swarm.runtimeReasons.modelUnavailable": modelUnavailable,
    "agentChat.swarm.runtimeReasons.modelConfigUnavailable":
      modelConfigUnavailable,
    "agentChat.swarm.runtimeReasons.workerUnavailable": workerUnavailable,
    "agentChat.swarm.runtimeReasons.timedOut": timedOut,
    "agentChat.swarm.runtimeReasons.collaborationIncomplete":
      collaborationIncomplete,
  }
  return messages[key] ?? key
}) as never

describe("swarm runtime reason view model", () => {
  it("maps model gateway failures to user-facing copy", () => {
    const reason =
      "worker reported failed status; llm provider error: upstream model service unavailable; structured role output is missing"
    const formatted = formatUserFacingRuntimeReason(reason, t)

    expect(formatted).toBe(modelUnavailable)
    expect(formatted).not.toContain("worker")
    expect(formatted).not.toContain("structured role output")
  })

  it("maps internal worker-only failures to a collaboration result gap", () => {
    const formatted = formatUserFacingRuntimeReason(
      "worker reported failed status; structured role output is missing",
      t
    )

    expect(formatted).toBe(collaborationIncomplete)
  })

  it("keeps already user-facing reasons", () => {
    expect(formatUserFacingRuntimeReason(modelUnavailableShort, t)).toBe(
      modelUnavailableShort
    )
    expect(isInternalRuntimeReason("structured role output is missing")).toBe(true)
  })
})

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}
