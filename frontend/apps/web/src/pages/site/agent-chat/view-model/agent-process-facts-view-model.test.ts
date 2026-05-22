import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getDispatchFacts } from "./agent-process-facts-view-model"

const planLabel = "Plan"
const planDetail = "complex task_graph"
const contextBudgetLabel = "Context budget"
const contextBudgetDetail = "12 history 3 materials"
const historyWindowLabel = "History window"
const historyWindowDetail = "recent 12 total 18"
const workspaceLabel = "Workspace materials"
const workspaceDetail = "partial 1/3"
const workspaceWarningLabel = "Material note"
const workspaceWarningDetail = "unsupported pdf"
const workspaceOmittedLabel = "Not in context"
const workspaceOmittedDetail = "2 omitted file-b, file-c"

const t = ((key: string, params?: Record<string, unknown>) => {
  if (key === "agentChat.thread.dispatchFacts.plan.label") return planLabel
  if (key === "agentChat.thread.dispatchFacts.plan.detail") {
    return `${params?.difficulty} ${params?.mode}`
  }
  if (key === "agentChat.thread.dispatchFacts.contextBudget.label") {
    return contextBudgetLabel
  }
  if (key === "agentChat.thread.dispatchFacts.contextBudget.detail") {
    return `${params?.history} history ${params?.materials} materials`
  }
  if (key === "agentChat.thread.dispatchFacts.historyWindow.label") {
    return historyWindowLabel
  }
  if (key === "agentChat.thread.dispatchFacts.historyWindow.truncated") {
    return `recent ${params?.recent} total ${params?.total}`
  }
  if (key === "agentChat.thread.dispatchFacts.workspace.label") {
    return workspaceLabel
  }
  if (key === "agentChat.thread.dispatchFacts.workspace.detail") {
    return `${params?.readiness} ${params?.readable}/${params?.total}`
  }
  if (key === "agentChat.thread.dispatchFacts.workspaceWarning.label") {
    return workspaceWarningLabel
  }
  if (key === "agentChat.thread.dispatchFacts.workspaceWarning.detail") {
    return String(params?.warning)
  }
  if (key === "agentChat.thread.dispatchFacts.workspaceOmitted.label") {
    return workspaceOmittedLabel
  }
  if (key === "agentChat.thread.dispatchFacts.workspaceOmitted.detail") {
    return `${params?.count} omitted ${params?.refs}`
  }
  return key
}) as never

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
        detail: planDetail,
        label: planLabel,
        tone: "success",
      },
      {
        detail: contextBudgetDetail,
        label: contextBudgetLabel,
        tone: "success",
      },
      {
        detail: historyWindowDetail,
        label: historyWindowLabel,
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
        detail: workspaceDetail,
        label: workspaceLabel,
        tone: "success",
      },
      {
        detail: workspaceWarningDetail,
        label: workspaceWarningLabel,
        tone: "warning",
      },
      {
        detail: workspaceOmittedDetail,
        label: workspaceOmittedLabel,
        tone: "warning",
      },
    ])
  })
})
