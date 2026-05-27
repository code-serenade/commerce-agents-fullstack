import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getDispatchFacts } from "./agent-process-facts-view-model"
import {
  dispatchFactTestLabels,
  t,
} from "./agent-process-facts-test-helpers"

describe("getDispatchFacts skill audit", () => {
  it("shows planned, requested, and succeeded non-search Skills", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "dispatch_planned",
        difficulty: "normal",
        planned_steps: [],
        primary_runtime_role: "ops",
        reasoning_summary: "",
        route_mode: "single_agent",
        selected_skills: ["ops_pricing_strategy", "search_competitor"],
        student_visible_explanation: "",
        support_runtime_roles: [],
      },
      {
        type: "skill_requested",
        reason: "autosuggest:ops.pricing",
        request_id: "req-ops",
        role: "ops",
        skill_id: "ops_pricing_strategy",
      },
      {
        output: {},
        request_id: "req-ops",
        role: "ops",
        skill_id: "ops_pricing_strategy",
        status: "success",
        summary: "pricing plan generated",
        type: "skill_result",
      },
    ]

    expect(getDispatchFacts(events, t)).toContainEqual({
      detail: dispatchFactTestLabels.skillAuditDetail,
      label: dispatchFactTestLabels.skillAuditLabel,
      tone: "success",
    })
  })

  it("warns when a requested non-search Skill has no result", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "dispatch_planned",
        difficulty: "normal",
        planned_steps: [],
        primary_runtime_role: "ops",
        reasoning_summary: "",
        route_mode: "single_agent",
        selected_skills: ["ops_inventory_plan"],
        student_visible_explanation: "",
        support_runtime_roles: [],
      },
      {
        type: "skill_requested",
        reason: "autosuggest:ops.inventory",
        request_id: "req-ops",
        role: "ops",
        skill_id: "ops_inventory_plan",
      },
    ]

    expect(getDispatchFacts(events, t)).toContainEqual({
      detail: dispatchFactTestLabels.skillAuditPending,
      label: dispatchFactTestLabels.skillAuditLabel,
      tone: "warning",
    })
  })

  it("keeps search Skills out of the generic Skill audit", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "dispatch_planned",
        difficulty: "normal",
        planned_steps: [],
        primary_runtime_role: "search",
        reasoning_summary: "",
        route_mode: "task_graph",
        selected_skills: ["search_competitor"],
        student_visible_explanation: "",
        support_runtime_roles: [],
      },
      {
        type: "skill_requested",
        reason: "autosuggest:search.competitor",
        request_id: "req-search",
        role: "search",
        skill_id: "search_competitor",
      },
    ]

    expect(getDispatchFacts(events, t)).not.toContainEqual(
      expect.objectContaining({
        label: dispatchFactTestLabels.skillAuditLabel,
      })
    )
    expect(getDispatchFacts(events, t)).toContainEqual({
      detail: "planned search_competitor requested search_competitor",
      label: dispatchFactTestLabels.searchAuditLabel,
      tone: "warning",
    })
  })
})
