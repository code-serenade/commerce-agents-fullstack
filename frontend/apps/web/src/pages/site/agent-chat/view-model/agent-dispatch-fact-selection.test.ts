import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getDispatchFacts } from "./agent-process-facts-view-model"
import {
  dispatchFactTestLabels,
  node,
  reported,
  t,
} from "./agent-process-facts-test-helpers"

const multiCheckReasoning = "needs multiple checks"
const searchClaimText = "competitor is discounting"
const searchSourceTitle = "source"

describe("selectDispatchFacts", () => {
  it("keeps role, search, skill, and material audits visible when dispatch facts exceed the compact limit", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "dispatch_planned",
        context_budget: {
          history_summary_available: true,
          history_total_message_count: 18,
          history_truncated: true,
          recent_history_message_count: 12,
          workspace_context_ref_count: 2,
          workspace_pack_item_count: 1,
        },
        difficulty: "complex",
        planned_steps: [],
        primary_runtime_role: "data",
        reasoning_summary: multiCheckReasoning,
        route_mode: "task_graph",
        selected_skills: ["ops_pricing_strategy", "search_competitor"],
        student_visible_explanation: "",
        support_runtime_roles: ["ops", "search"],
      },
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: "diagnose campaign",
          nodes: [node("data", "data"), node("ops", "ops"), node("search", "search")],
        },
      },
      reported("data", "data"),
      reported("ops", "ops"),
      {
        type: "skill_result",
        output: {
          evidence_packet: {
            confidence: 0.72,
            gaps: [],
            search_strength: "standard",
            sources: [{ title: searchSourceTitle }],
            usable_claims: [{ text: searchClaimText }],
          },
        },
        request_id: "request-1",
        role: "search",
        skill_id: "search_competitor",
        status: "ok",
      },
      {
        type: "skill_result",
        output: {},
        request_id: "request-2",
        role: "ops",
        skill_id: "ops_pricing_strategy",
        status: "success",
      },
      {
        type: "context_loaded",
        contextRefs: ["conversation_file:file-a", "conversation_file:file-b"],
        knowledge_count: 0,
        product_count: 0,
        source_count: 2,
      },
      {
        type: "workspace_context_pack_loaded",
        items: [
          { id: "file-a", refKey: "conversation_file:file-a", text: "readable text" },
        ],
        readiness: "ready",
      },
      {
        count: 1,
        kinds: ["preference"],
        type: "memory_loaded",
      },
      {
        message_count: 12,
        relevant_older_count: 2,
        total_message_count: 18,
        type: "conversation_context_loaded",
      },
    ]

    const facts = getDispatchFacts(events, t)

    expect(facts).toHaveLength(6)
    expect(facts).toContainEqual({
      detail: "planned Data, Ops, Search executed Data, Ops, Search",
      label: dispatchFactTestLabels.roleAuditLabel,
      tone: "success",
    })
    expect(facts).toContainEqual({
      detail: dispatchFactTestLabels.searchAuditDetail,
      label: dispatchFactTestLabels.searchAuditLabel,
      tone: "success",
    })
    expect(facts).toContainEqual({
      detail: "planned 1 requested 0 succeeded 1 failed 0 pending 0",
      label: dispatchFactTestLabels.skillAuditLabel,
      tone: "success",
    })
    expect(facts).toContainEqual({
      detail:
        "missing pack 1 selected 2 packed 1 readable 1 missing 1 omitted 0 unreadable 0 warnings 0 readiness ready",
      label: dispatchFactTestLabels.materialAuditLabel,
      tone: "warning",
    })
  })
})
