import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getDispatchWorkbenchModel } from "./agent-dispatch-workbench-view-model"
import { t } from "./agent-process-facts-test-helpers"

const task = "diagnose campaign"
const capabilityCheck = textFromCodes(33021, 21147, 26680, 23545)
const taskPlanWithRawDifficulty = `${textFromCodes(
  20219,
  21153,
  23433,
  25490
)} complex ${textFromCodes(38590, 24230)}`
const contextReadWarning = `${textFromCodes(
  36164,
  26009,
  35835,
  21462,
  25552,
  31034
)} no active workspace context sources were found`

describe("getDispatchWorkbenchModel sanitization", () => {
  it("hides no-workspace and capability check details from process steps", () => {
    const events: AgentRuntimeEvent[] = [
      {
        difficulty: "complex",
        planned_steps: [
          {
            detail: contextReadWarning,
            id: "context-check",
            title: capabilityCheck,
          },
        ],
        primary_runtime_role: "data",
        reasoning_summary: taskPlanWithRawDifficulty,
        route_mode: "task_graph",
        selected_skills: [],
        student_visible_explanation: "",
        support_runtime_roles: ["ops"],
        type: "dispatch_planned",
      },
    ]

    const model = getDispatchWorkbenchModel(
      { isStreaming: false, runtimeEvents: events, task },
      t
    )
    const visible = JSON.stringify({
      facts: model.facts,
      previewLines: model.previewLines,
      steps: model.steps,
      summary: model.summary,
    }).toLowerCase()

    expect(visible).not.toContain("no active workspace")
    expect(visible).not.toContain("complex")
    expect(visible).not.toContain(capabilityCheck)
    expect(model.steps[0]?.detail).toBe("user facing detail")
  })
})

function textFromCodes(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
