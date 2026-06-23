import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getDispatchFacts } from "./agent-process-facts-view-model"
import { t } from "./agent-process-facts-test-helpers"

describe("getDispatchFacts user-facing text", () => {
  it("keeps dispatch facts readable without exposing runtime internals", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "dispatch_planned",
        difficulty: "complex",
        planned_steps: [],
        primary_runtime_role: "data",
        reasoning_summary: "route_mode task_graph selected_skills search_trends",
        route_mode: "task_graph",
        selected_skills: ["creative_title_ctr_scorer", "search_trends"],
        student_visible_explanation: "",
        support_runtime_roles: ["creative", "design"],
      },
      {
        type: "route_decided",
        primary_runtime_role: "data",
        route_mode: "task_graph",
        strategy_id: "learning",
        support_runtime_roles: ["creative"],
      },
      {
        type: "skill_result",
        output: {
          evidence_packet: {
            confidence: 0,
            gaps: [
              "external search provider request failed: http://internal/search",
            ],
            sources: [],
            usable_claims: [],
          },
        },
        request_id: "req-search",
        role: "search",
        skill_id: "search_trends",
        status: "deferred",
        summary: "search_evidence_trace: skill=search_trends",
      },
      {
        output: {},
        request_id: "req-title",
        role: "creative",
        skill_id: "creative_title_ctr_scorer",
        status: "success",
        summary: "worker.assignment.input.data",
        type: "skill_result",
      },
      {
        count: 1,
        kinds: ["worker.memory"],
        records: [
          {
            char_count: 32,
            key: "worker.assignment.input.data",
            kind: "strategy.internal",
            tags: ["worker", "search_trends"],
          },
        ],
        type: "memory_loaded",
      },
      {
        type: "rag_context_loaded",
        degraded: true,
        degraded_reason:
          "external search provider request failed: http://internal/search",
        source_count: 0,
      },
      { type: "output", content: "final answer" },
    ]

    const text = JSON.stringify(getDispatchFacts(events, t)).toLowerCase()

    expect(text).not.toContain("worker")
    expect(text).not.toContain("skill_id")
    expect(text).not.toContain("creative_title_ctr_scorer")
    expect(text).not.toContain("search_trends")
    expect(text).not.toContain("route_mode")
    expect(text).not.toContain("strategy")
    expect(text).not.toContain("http://")
    expect(text).not.toContain("search_evidence")
  })
})
