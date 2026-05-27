import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getDispatchFacts } from "./agent-process-facts-view-model"
import {
  dispatchFactTestLabels,
  t,
} from "./agent-process-facts-test-helpers"

describe("getDispatchFacts search audit", () => {
  it("shows executed search skills with evidence packet quality", () => {
    const sourceTitle = ["Cold", "brew", "price", "reference"].join(" ")
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
        type: "skill_result",
        request_id: "req-1",
        role: "search",
        skill_id: "search_competitor",
        status: "success",
        summary: "found competitor prices",
        output: {
          evidence_packet: {
            confidence: 0.72,
            gaps: [],
            search_strength: "standard",
            sources: [
              {
                title: sourceTitle,
                url: "https://example.com/cold-brew-price",
              },
            ],
            usable_claims: ["competitor prices cluster at 12-19 CNY"],
          },
        },
      },
    ]

    expect(getDispatchFacts(events, t)).toContainEqual({
      detail: dispatchFactTestLabels.searchAuditDetail,
      label: dispatchFactTestLabels.searchAuditLabel,
      tone: "success",
    })
  })

  it("warns when search is only requested but no result is returned yet", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "dispatch_planned",
        difficulty: "normal",
        planned_steps: [],
        primary_runtime_role: "search",
        reasoning_summary: "",
        route_mode: "task_graph",
        selected_skills: ["search_platform_policy"],
        student_visible_explanation: "",
        support_runtime_roles: [],
      },
      {
        type: "skill_requested",
        reason: "autosuggest:search.platform_policy",
        request_id: "req-1",
        role: "search",
        skill_id: "search_platform_policy",
      },
    ]

    expect(getDispatchFacts(events, t)).toContainEqual({
      detail: dispatchFactTestLabels.searchAuditPending,
      label: dispatchFactTestLabels.searchAuditLabel,
      tone: "warning",
    })
  })
})
