import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getDispatchWorkbenchModel } from "./agent-dispatch-workbench-view-model"
import {
  dispatchFactTestLabels,
  node,
  reported,
  t,
} from "./agent-process-facts-test-helpers"

const task = "diagnose campaign"
const multiCheckReasoning = "needs multiple checks"
const searchClaimText = "competitor is discounting"
const searchSourceTitle = "source"
const graphId = "graph-1"
const finalStepTitle = "design reviewer"
const internalWorkerTitle = "worker.design"
const machineStepTitle = "task_node_started"

describe("getDispatchWorkbenchModel", () => {
  it("keeps every selected audit fact in the preview document", () => {
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
        selected_skills: ["search_competitor"],
        student_visible_explanation: "",
        support_runtime_roles: ["ops", "search"],
      },
      {
        type: "task_graph_planned",
        graph: {
          graph_id: graphId,
          root_task: task,
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

    const model = getDispatchWorkbenchModel(
      { isStreaming: false, runtimeEvents: events, task },
      t
    )

    expect(model.facts).toHaveLength(6)
    expect(model.previewLines).toContain(
      `${dispatchFactTestLabels.searchAuditLabel}: ${dispatchFactTestLabels.searchAuditDetail}`
    )
    expect(model.previewLines).toContain(
      "Material audit: missing pack 1 selected 2 packed 1 readable 1 missing 1 omitted 0 unreadable 0 warnings 0 readiness ready"
    )
  })

  it("keeps all planned steps in the preview document", () => {
    const roleIds = [
      "data",
      "ops",
      "search",
      "creative",
      "accounting",
      "web",
      "service",
      "design",
    ]
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: graphId,
          root_task: task,
          nodes: roleIds.map((roleId, index) =>
            index === roleIds.length - 1
              ? { ...node(roleId, roleId), title: finalStepTitle }
              : node(roleId, roleId)
          ),
        },
      },
    ]

    const model = getDispatchWorkbenchModel(
      { isStreaming: false, runtimeEvents: events, task },
      t
    )

    expect(model.steps).toHaveLength(roleIds.length)
    expect(model.previewLines).toContain(
      "08 presentation advice: design objective"
    )
    expect(model.agents.map((agent) => agent.nodeId)).toContain("data")
    expect(model.agents.map((agent) => agent.nodeId)).toContain("ops")
    expect(model.agents.at(-1)?.nodeId).toBe("delivery")
  })

  it("does not expose i18n keys in the preview document", () => {
    const testT = t as unknown as (
      key: string,
      params?: Record<string, unknown>
    ) => string
    const incompleteT = ((key: string, params?: Record<string, unknown>) => {
      if (key === "agentChat.thread.dispatchFacts.materialAudit.label") return key
      if (key === "agentChat.thread.dispatchFacts.materialAudit.detail") return key
      if (key === "agentChat.thread.dispatchFacts.materialAudit.summary.empty") {
        return "no material"
      }
      if (key === "agentChat.thread.dispatchPanel.preview.fact") {
        return `${params?.label}: ${params?.detail}`
      }
      if (key === "agentChat.thread.dispatchPanel.preview.fallbackFactLabel") {
        return "Task materials"
      }
      if (key === "agentChat.thread.dispatchPanel.preview.fallbackFactDetail") {
        return "Task material status synced"
      }
      return testT(key, params)
    }) as never
    const events: AgentRuntimeEvent[] = [
      {
        readiness: "empty",
        type: "workspace_context_pack_loaded",
      },
    ]

    const model = getDispatchWorkbenchModel(
      { isStreaming: false, runtimeEvents: events, task },
      incompleteT
    )

    expect(model.previewLines.join("\n")).not.toContain("agentChat.")
    expect(model.previewLines).toContain("Task materials: Task material status synced")
  })

  it("does not expose internal runtime terms in the preview document", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "dispatch_planned",
        difficulty: "complex",
        planned_steps: [
          {
            detail: "worker.design should use task_graph selected_skills",
            id: "worker-design",
            title: internalWorkerTitle,
          },
        ],
        primary_runtime_role: "design",
        reasoning_summary: "route_mode task_graph selected_skills",
        route_mode: "task_graph",
        selected_skills: ["design_material_spec"],
        student_visible_explanation: "",
        support_runtime_roles: ["design"],
      },
    ]

    const model = getDispatchWorkbenchModel(
      { isStreaming: false, runtimeEvents: events, task },
      t
    )
    const preview = model.previewLines.join("\n").toLowerCase()
    const steps = JSON.stringify(model.steps).toLowerCase()

    expect(preview).not.toContain("worker")
    expect(preview).not.toContain("selected_skills")
    expect(preview).not.toContain("route_mode")
    expect(preview).not.toContain("task_graph")
    expect(steps).not.toContain("worker")
    expect(steps).not.toContain("selected_skills")
    expect(steps).not.toContain("task_graph")
  })

  it("compacts machine status and event names in the preview document", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "dispatch_planned",
        difficulty: "complex",
        planned_steps: [
          {
            detail: "validation_error mark_agent_task_started request_id=req-1",
            id: "internal-step",
            title: machineStepTitle,
          },
        ],
        primary_runtime_role: "design",
        reasoning_summary: "failure_kind=json_template_validation",
        route_mode: "task_graph",
        selected_skills: ["design_material_spec"],
        student_visible_explanation: "",
        support_runtime_roles: ["design"],
      },
    ]

    const model = getDispatchWorkbenchModel(
      { isStreaming: false, runtimeEvents: events, task },
      t
    )
    const preview = model.previewLines.join("\n").toLowerCase()
    const steps = JSON.stringify(model.steps).toLowerCase()

    expect(preview).not.toContain("validation_error")
    expect(preview).not.toContain("mark_agent_task_started")
    expect(preview).not.toContain("request_id")
    expect(preview).not.toContain("task_node_started")
    expect(preview).not.toContain("failure_kind")
    expect(preview).not.toContain("json_template_validation")
    expect(steps).not.toContain("validation_error")
    expect(steps).not.toContain("request_id")
  })

  it("shows only the relevant role in quick dispatch instead of every preset agent", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "route_decided",
        primary_runtime_role: "chat",
        route_mode: "chat_bypass",
        support_runtime_roles: [],
      },
      {
        content: "answer",
        type: "output",
      },
    ]

    const model = getDispatchWorkbenchModel(
      { isStreaming: false, runtimeEvents: events, task },
      t
    )

    expect(model.agents.map((agent) => agent.nodeId)).toEqual(["ops"])
  })
})
