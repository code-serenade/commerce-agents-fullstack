import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getDispatchFacts } from "./agent-process-facts-view-model"
import {
  dispatchFactTestLabels,
  node,
  reported,
  t,
} from "./agent-process-facts-test-helpers"

const textFromCodes = (codes: number[]) =>
  String.fromCharCode(...codes)

const thermosEntity = textFromCodes([20415, 25658, 20445, 28201, 26479])
const retainedBundleDecision = textFromCodes([
  23458, 25143, 20915, 23450, 20445, 30041, 39640, 27611, 21033, 22871, 39184,
])

describe("getDispatchFacts", () => {
  it("shows truncated conversation history window from backend dispatch budget", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "dispatch_planned",
        context_budget: {
          approximate_input_chars: 4200,
          explicit_context_ref_count: 0,
          explicit_product_count: 1,
          history_older_summary_available: true,
          history_summary_available: true,
          history_total_message_count: 18,
          history_truncated: true,
          memory_record_count: 0,
          recent_history_message_count: 12,
          workspace_context_ref_count: 2,
          workspace_pack_item_count: 0,
        },
        difficulty: "complex",
        planned_steps: [],
        primary_runtime_role: "data",
        reasoning_summary: "",
        route_mode: "task_graph",
        selected_skills: [],
        student_visible_explanation: "",
        support_runtime_roles: [],
      },
    ]

    expect(getDispatchFacts(events, t)).toEqual([
      {
        detail: dispatchFactTestLabels.planDetail,
        label: dispatchFactTestLabels.planLabel,
        tone: "success",
      },
      {
        detail: dispatchFactTestLabels.contextBudgetDetail,
        label: dispatchFactTestLabels.contextBudgetLabel,
        tone: "success",
      },
      {
        detail: dispatchFactTestLabels.historyWindowDetail,
        label: dispatchFactTestLabels.historyWindowLabel,
        tone: "success",
      },
    ])
  })

  it("shows workspace material warnings and omitted refs from context pack events", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "workspace_context_pack_loaded",
        items: [
          { id: "file-a", text: "readable text" },
          { id: "file-b" },
          { id: "file-c" },
        ],
        omitted: ["file-b", "file-c"],
        readiness: "partial",
        warnings: ["unsupported pdf"],
      },
    ]

    expect(getDispatchFacts(events, t)).toEqual([
      {
        detail:
          "omitted 2 selected 0 packed 3 readable 1 missing 0 omitted 2 unreadable 2 warnings 1 readiness partial",
        label: dispatchFactTestLabels.materialAuditLabel,
        tone: "warning",
      },
      {
        detail: dispatchFactTestLabels.workspaceDetail,
        label: dispatchFactTestLabels.workspaceLabel,
        tone: "success",
      },
      {
        detail: dispatchFactTestLabels.workspaceWarningDetail,
        label: dispatchFactTestLabels.workspaceWarningLabel,
        tone: "warning",
      },
      {
        detail: dispatchFactTestLabels.workspaceOmittedDetail,
        label: dispatchFactTestLabels.workspaceOmittedLabel,
        tone: "warning",
      },
    ])
  })

  it("shows selected material refs against readable workspace pack items", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "context_loaded",
        contextRefs: [
          "conversation_file:file-a",
          "product:product-a",
          "product_material:material-a",
        ],
        knowledge_count: 0,
        product_count: 1,
        refs: ["conversation_file:file-a"],
        source_count: 2,
      },
      {
        type: "workspace_context_pack_loaded",
        items: [
          { id: "file-a", refKey: "conversation_file:file-a", text: "readable text" },
          { id: "material-a", ref_key: "product_material:material-a", text: "rows" },
        ],
        readiness: "ready",
      },
    ]

    expect(getDispatchFacts(events, t)).toContainEqual({
      detail:
        "ready selected 2 packed 2 readable 2 missing 0 omitted 0 unreadable 0 warnings 0 readiness ready",
      label: dispatchFactTestLabels.materialAuditLabel,
      tone: "success",
    })
  })

  it("uses safe workspace pack counts without requiring item bodies", () => {
    const hiddenPackSummary = "hidden pack summary"
    const events: AgentRuntimeEvent[] = [
      {
        type: "context_loaded",
        contextRefs: ["conversation_file:file-a", "product_material:material-a"],
        knowledge_count: 0,
        product_count: 0,
        source_count: 2,
      },
      {
        type: "workspace_context_pack_loaded",
        itemCount: 2,
        readableItemCount: 2,
        readiness: "ready",
        summary: hiddenPackSummary,
      },
    ]

    const facts = getDispatchFacts(events, t)

    expect(facts).toContainEqual({
      detail:
        "ready selected 2 packed 2 readable 2 missing 0 omitted 0 unreadable 0 warnings 0 readiness ready",
      label: dispatchFactTestLabels.materialAuditLabel,
      tone: "success",
    })
    const serialized = JSON.stringify(facts)
    expect(serialized).not.toContain(hiddenPackSummary)
  })

  it("warns when selected material refs have not reached the workspace pack", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "context_loaded",
        contextRefs: ["conversation_file:file-a", "product:product-a"],
        knowledge_count: 0,
        product_count: 1,
        source_count: 1,
      },
    ]

    expect(getDispatchFacts(events, t)).toContainEqual({
      detail:
        "pending pack selected 1 packed 0 readable 0 missing 1 omitted 0 unreadable 0 warnings 0 readiness pending",
      label: dispatchFactTestLabels.materialAuditLabel,
      tone: "warning",
    })
  })

  it("warns when selected material refs are missing from a loaded workspace pack", () => {
    const events: AgentRuntimeEvent[] = [
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
    ]

    expect(getDispatchFacts(events, t)).toContainEqual({
      detail:
        "missing pack 1 selected 2 packed 1 readable 1 missing 1 omitted 0 unreadable 0 warnings 0 readiness ready",
      label: dispatchFactTestLabels.materialAuditLabel,
      tone: "warning",
    })
  })

  it("shows memory record summaries without memory content", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "memory_loaded",
        count: 2,
        kinds: ["conversation_fact"],
        records: [
          {
            char_count: 86,
            key: "memory_card:facts",
            kind: "conversation_fact",
            tags: ["memory_card", `entity:${thermosEntity}`],
          },
          {
            charCount: 42,
            key: "memory_card:decisions",
            kind: "conversation_fact",
            tags: ["memory_card", "decision"],
            truncated: true,
          },
        ],
      },
    ]

    const facts = getDispatchFacts(events, t)

    expect(facts).toContainEqual({
      detail: dispatchFactTestLabels.memoryDetailWithRecords,
      label: dispatchFactTestLabels.memoryLabel,
      tone: "success",
    })
    expect(JSON.stringify(facts)).not.toContain(retainedBundleDecision)
  })

  it("shows planned and executed role coverage for task graph events", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: "diagnose campaign",
          nodes: [
            node("data", "data"),
            node("ops", "ops"),
            node("ops_final", "ops"),
          ],
        },
      },
      reported("data", "data"),
      reported("ops", "ops"),
      reported("ops_final", "ops"),
    ]

    expect(getDispatchFacts(events, t)).toContainEqual({
      detail: dispatchFactTestLabels.roleAuditDetail,
      label: dispatchFactTestLabels.roleAuditLabel,
      tone: "success",
    })
  })

  it("warns when a planned role has not produced a report", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: "diagnose campaign",
          nodes: [
            node("data", "data"),
            node("ops", "ops"),
          ],
        },
      },
      reported("data", "data"),
    ]

    expect(getDispatchFacts(events, t)).toContainEqual({
      detail: dispatchFactTestLabels.roleAuditWarning,
      label: dispatchFactTestLabels.roleAuditLabel,
      tone: "warning",
    })
  })
})
