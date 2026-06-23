import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import {
  getAgentProcessBadge,
  getAgentProcessSummary,
  getAgentProcessKind,
  getAgentProcessProgress,
  getProcessStepRows,
} from "./agent-process-kind-view-model"

const t = ((key: string, params?: Record<string, unknown>) => {
  if (key === "agentChat.thread.processKinds.quick_dispatch.progress") {
    return `${params?.completed}/${params?.total} dispatch`
  }
  if (key === "agentChat.thread.processKinds.quick_dispatch.stageFinished") {
    return "quick finished"
  }
  if (key === "agentChat.thread.processKinds.multi_agent.progress") {
    return `${params?.completed}/${params?.total} multi`
  }
  if (key === "agentChat.thread.processKinds.multi_agent.stageRunning") {
    return "multi running"
  }
  if (key === "agentChat.thread.processKinds.multi_agent.stageFinished") {
    return "multi finished"
  }
  if (key === "agentChat.thread.processKinds.multi_agent.badge") {
    return "organizing"
  }
  if (key === "agentChat.thread.processKinds.completed.badge") {
    return "done"
  }
  if (key === "agentChat.thread.steps.readTarget") return "read target"
  if (key === "agentChat.thread.steps.contextLoaded") {
    return `${params?.products} products ${params?.knowledge} knowledge`
  }
  if (key === "agentChat.thread.steps.dispatchJudge") return "dispatch judge"
  if (key === "agentChat.thread.steps.dispatchRoute") {
    return `${params?.mode} ${params?.role}`
  }
  if (key === "agentChat.thread.dispatchPanel.routeModes.chat_bypass") {
    return "quick answer"
  }
  if (key === "agentChat.thread.dispatchPanel.routeModes.task_graph") {
    return "multi-step"
  }
  if (key === "agentChat.thread.dispatchPanel.routeModes.single_agent") {
    return "main angle"
  }
  if (key === "agentChat.thread.runtimeText.routeFallback") {
    return "current method"
  }
  if (key === "agentChat.thread.steps.runtimeAction") return "runtime action"
  if (key === "agentChat.thread.steps.skillResult") {
    return `${params?.count} references checked ${params?.status ?? ""}`.trim()
  }
  if (key === "agentChat.thread.runtimeText.referenceReady") return "ready"
  if (key === "agentChat.thread.runtimeText.referenceLimited") return "limited"
  if (key === "agentChat.thread.runtimeText.referencePending") return "pending"
  if (key === "agentChat.thread.runtimeText.referenceSynced") return "synced"
  if (key === "agentChat.thread.steps.userFacingTitles.data") {
    return "check available info"
  }
  if (key === "agentChat.thread.steps.answerReady") return "answer ready"
  if (key === "agentChat.thread.steps.answerFinished") return "answer finished"
  if (key === "agentChat.thread.roleFallbackTitles.chat") return "direct answer"
  return key
}) as never
const dataTitle = ["Data", "diagnosis"].join(" ")
const dataObjective = ["Check", "data"].join(" ")
const opsTitle = ["Ops", "plan"].join(" ")
const opsObjective = ["Plan", "execution"].join(" ")
const readContextTitle = ["Read", "context"].join(" ")
const readSelectedMaterialsDetail = ["Read", "selected", "materials"].join(" ")
const planFromBackendReason = ["Plan", "from", "backend"].join(" ")
const improveProductTitleTask = ["improve", "product", "title"].join(" ")
const graphNodeObjectiveSuffix = "check"
const finalAnswerContent = ["Final", "answer", "body"].join(" ")

describe("agent process kind", () => {
  it("keeps chat_bypass as quick dispatch with real route and skill steps", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "context_loaded",
        knowledge_count: 2,
        product_count: 1,
      },
      {
        type: "route_decided",
        route_mode: "chat_bypass",
        primary_runtime_role: "chat",
        support_runtime_roles: [],
      },
      {
        type: "skill_result",
        request_id: "skill-1",
        skill_id: "coordination_agent_handoff",
        role: "chat",
        status: "success",
      },
      { type: "finished" },
    ]

    expect(getAgentProcessKind(events)).toBe("quick_dispatch")
    expect(getProcessStepRows(events, false, t)).toEqual([
      ["read target", "1 products 2 knowledge"],
      ["dispatch judge", "quick answer direct answer"],
      ["runtime action", "1 references checked ready"],
      ["answer ready", "answer finished"],
    ])
    const quickDispatchProgressLabel = ["4/4", "dispatch"].join(" ")

    expect(getAgentProcessProgress({ isStreaming: false, runtimeEvents: events }, t))
      .toEqual({
        completed: 4,
        label: quickDispatchProgressLabel,
        percent: 100,
        stage: "quick finished",
        total: 4,
      })
  })

  it("classifies a graph with multiple non-chat nodes as multi-agent", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: ["campaign", "plan"].join(" "),
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: "worker.data",
              title: dataTitle,
              objective: dataObjective,
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
            {
              id: "ops",
              role: "ops",
              worker_id: "worker.ops",
              title: opsTitle,
              objective: opsObjective,
              input_refs: ["data"],
              acceptance: [],
              max_attempts: 1,
            },
          ],
        },
      },
    ]

    expect(getAgentProcessKind(events)).toBe("multi_agent")
    const multiAgentProgressLabel = ["1/3", "multi"].join(" ")

    expect(
      getAgentProcessProgress({
        completedCount: 1,
        isStreaming: true,
        runtimeEvents: events,
        totalCount: 3,
      }, t)
    ).toEqual({
      completed: 1,
      label: multiAgentProgressLabel,
      percent: 33,
      stage: "multi running",
      total: 3,
    })
  })

  it("keeps chat-only skill events as quick dispatch when route metadata is absent", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "skill_result",
        request_id: "skill-chat",
        skill_id: "coordination_agent_handoff",
        role: "chat",
        status: "success",
      },
      { type: "finished" },
    ]

    expect(getAgentProcessKind(events)).toBe("quick_dispatch")
  })

  it("uses backend dispatch plan for process kind, summary, and steps", () => {
    const studentExplanation = ["Backend", "planned", "collaboration"].join(" ")
    const events: AgentRuntimeEvent[] = [
      {
        type: "dispatch_planned",
        context_budget: {
          memory_record_count: 2,
          history_summary_available: true,
          history_older_summary_available: false,
          recent_history_message_count: 4,
          history_total_message_count: 4,
          history_truncated: false,
          explicit_product_count: 1,
          explicit_context_ref_count: 1,
          workspace_context_ref_count: 0,
          workspace_pack_item_count: 1,
          approximate_input_chars: 2400,
        },
        difficulty: "complex",
        planned_steps: [
          {
            id: "context",
            title: readContextTitle,
            detail: readSelectedMaterialsDetail,
          },
          {
            id: "data",
            title: dataTitle,
            detail: dataObjective,
          },
        ],
        primary_runtime_role: "data",
        reasoning_summary: planFromBackendReason,
        route_mode: "task_graph",
        selected_skills: [],
        student_visible_explanation: studentExplanation,
        support_runtime_roles: ["ops"],
      },
    ]

    expect(getAgentProcessKind(events)).toBe("multi_agent")
    expect(getAgentProcessSummary(events, "task", t)).toBe(studentExplanation)
    expect(getProcessStepRows(events, false, t)).toEqual([
      [readContextTitle, readSelectedMaterialsDetail],
      ["check available info", dataObjective],
    ])
  })

  it("treats final output as completed even when a node failed before summary", () => {
    const completedProgressLabel = ["5/5", "multi"].join(" ")
    const events: AgentRuntimeEvent[] = [
      createFiveStepGraph(),
      {
        type: "task_node_passed",
        node_id: "data",
      },
      {
        type: "task_node_passed",
        node_id: "ops",
      },
      {
        type: "task_node_failed",
        node_id: "design",
        reason: "design role output lacks design-specific content",
      },
      {
        type: "output",
        content: finalAnswerContent,
      },
    ]

    expect(getAgentProcessKind(events)).toBe("multi_agent")
    expect(
      getAgentProcessProgress({
        completedCount: 4,
        isStreaming: false,
        runtimeEvents: events,
        totalCount: 5,
      }, t)
    ).toEqual({
      completed: 5,
      label: completedProgressLabel,
      percent: 100,
      stage: "multi finished",
      total: 5,
    })
    expect(getAgentProcessBadge(events, t)).toBe("done")
  })

  it("keeps multi-agent badge as in-progress before final output", () => {
    const events: AgentRuntimeEvent[] = [createFiveStepGraph()]

    expect(getAgentProcessBadge(events, t)).toBe("organizing")
  })

})

function createFiveStepGraph(): AgentRuntimeEvent {
  return {
    type: "task_graph_planned",
    graph: {
      graph_id: "graph-5",
      root_task: improveProductTitleTask,
      nodes: ["data", "ops", "creative", "design"].map((role) => {
        const title = role
        return {
          id: role,
          role,
          worker_id: `worker.${role}`,
          title,
          objective: [role, graphNodeObjectiveSuffix].join(" "),
          input_refs: [],
          acceptance: [],
          max_attempts: 1,
        }
      }),
    },
  }
}
