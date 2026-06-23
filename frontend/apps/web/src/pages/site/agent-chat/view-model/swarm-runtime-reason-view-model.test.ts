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

  it("maps internal role quality checks to a collaboration result gap", () => {
    const formatted = formatUserFacingRuntimeReason(
      "design role output lacks design-specific content",
      t
    )

    expect(formatted).toBe(collaborationIncomplete)
    expect(formatted).not.toContain("design role")
  })

  it("maps route, skill, search evidence, and adapter internals to a collaboration result gap", () => {
    const reason = [
      "route_mode=task_graph",
      "selected_skills=search_trends",
      "search_evidence_trace missing",
      "adapter request failed https://internal.example/search",
    ].join("; ")
    const formatted = formatUserFacingRuntimeReason(reason, t)

    expect(formatted).toBe(collaborationIncomplete)
    expect(formatted).not.toContain("route_mode")
    expect(formatted).not.toContain("task_graph")
    expect(formatted).not.toContain("selected_skills")
    expect(formatted).not.toContain("search_evidence")
    expect(formatted).not.toContain("adapter")
    expect(formatted).not.toContain("https://")
    expect(isInternalRuntimeReason(reason)).toBe(true)
  })

  it("maps raw worker and skill permission internals to a collaboration result gap", () => {
    const reason =
      "worker.data failed because skill 'coordination_create_task' is not allowed for role 'engineering'"
    const formatted = formatUserFacingRuntimeReason(reason, t)

    expect(formatted).toBe(collaborationIncomplete)
    expect(formatted).not.toContain("worker.data")
    expect(formatted).not.toContain("skill")
    expect(formatted).not.toContain("coordination_create_task")
    expect(formatted).not.toContain("engineering")
    expect(isInternalRuntimeReason(reason)).toBe(true)
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
