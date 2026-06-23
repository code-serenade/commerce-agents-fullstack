import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import {
  getAgentProcessSummary,
  getProcessStepRows,
} from "./agent-process-kind-view-model"

const softenedDispatchExplanation = "softened dispatch explanation"
const planFromBackendReason = ["Plan", "from", "backend"].join(" ")

const t = ((key: string) => {
  if (key === "agentChat.thread.processKinds.sanitized.dispatchCollaboration") {
    return softenedDispatchExplanation
  }
  if (key === "agentChat.thread.processKinds.sanitized.multiCheck") {
    return "multiple checks"
  }
  if (key === "agentChat.thread.processKinds.sanitized.multiReference") {
    return "multiple references"
  }
  if (key === "agentChat.thread.steps.userFacingDetail") {
    return "user facing detail"
  }
  if (key === "agentChat.thread.steps.userFacingTitles.design") {
    return "presentation advice"
  }
  return key
}) as never

describe("agent process user-facing text", () => {
  it("softens backend dispatch explanations before showing them", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "dispatch_planned",
        difficulty: "complex",
        planned_steps: [],
        primary_runtime_role: "data",
        reasoning_summary: planFromBackendReason,
        route_mode: "task_graph",
        selected_skills: [],
        student_visible_explanation: textFromCodes([
          36825, 20010, 38382, 39064, 20250, 25286, 32473, 32, 52, 32, 20010,
          23703, 20301, 47, 27493, 39588, 21327, 20316, 65292, 36807, 31243,
          26356, 38271, 20294, 32467, 26524, 20250, 26356, 23436, 25972,
          12290,
        ]),
        support_runtime_roles: ["ops"],
      },
    ]

    expect(getAgentProcessSummary(events, "task", t)).toBe(
      softenedDispatchExplanation
    )
  })

  it("compacts backend planned role prompts into user-facing steps", () => {
    const internalDetail = [
      textFromCodes([
        20316, 20026, 35774, 35745, 24072, 65292, 25226, 21334, 28857,
        12289, 20449, 20219, 35777, 26126, 12289, 39029, 38754, 23618,
        32423, 25215, 25509, 25104, 21487, 27979, 35797, 26041, 26696,
        65307,
      ]),
      "skill:search_trends ",
      textFromCodes([23578, 26410, 20135, 20986, 21487, 29992, 32467, 26524, 65292]),
      "worker.assignment.input.data",
    ].join("")
    const events: AgentRuntimeEvent[] = [
      {
        type: "dispatch_planned",
        difficulty: "complex",
        planned_steps: [
          {
            detail: internalDetail,
            id: "design",
            title: textFromCodes([20307, 39564, 35774, 35745]),
          },
        ],
        primary_runtime_role: "data",
        reasoning_summary: "",
        route_mode: "task_graph",
        selected_skills: [],
        student_visible_explanation: "",
        support_runtime_roles: ["design"],
      },
    ]

    expect(getProcessStepRows(events, false, t)).toEqual([
      ["presentation advice", "user facing detail"],
    ])
  })
})

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}
