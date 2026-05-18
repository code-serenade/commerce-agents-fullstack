import { describe, expect, it } from "vitest"
import { BarChart3 } from "lucide-react"
import type { AgentRuntimeEvent } from "@/api"
import { getRuntimeNodeStates } from "./swarm-runtime-state-view-model"
import { hasBlockingTaskFailure, hasBlockingWorkspaceFailure } from "./thread-boundary-state-view-model"
import { getInlineAgents, getInlineProcessStage } from "./thread-inline-view-model"
import { formatTimelineEvent } from "./swarm-timeline-view-model"
import { getWorkspacePersistenceCard } from "./swarm-workspace-view-model"

const t = ((key: string, options?: Record<string, string>) => {
  if (key === "agentChat.thread.processStage.finished") return "finished"
  if (key === "agentChat.thread.processStage.failed") return "failed"
  if (key === "agentChat.thread.inlineState.done") return "done"
  if (key === "agentChat.thread.agents.data.name") return "data"
  if (key === "agentChat.thread.agents.data.summary") return "data summary"
  if (key === "agentChat.thread.agents.service.name") return "service"
  if (key === "agentChat.thread.agents.service.summary") return "service summary"
  if (key === "agentChat.thread.agents.delivery.name") return "delivery"
  if (key === "agentChat.thread.agents.delivery.summary") return "delivery summary"
  if (key === "agentChat.swarm.timeline.events.nodeBoundary") return "node boundary"
  if (key === "agentChat.swarm.timeline.events.workspaceTaskBoundary") return "workspace boundary"
  if (key === "agentChat.swarm.cards.workspace.taskBoundaryTitle") return "task boundary"
  if (key === "agentChat.swarm.cards.workspace.taskBoundaryDescription") return "task boundary description"
  if (key === "agentChat.swarm.cards.workspace.runTaskLine") return `${options?.id}:${options?.status}`
  return key
}) as never

const coffeeCategoryTask = [
  textFromCodes([35831, 20998, 26512]),
  textFromCodes([21654, 21857, 31867]),
  textFromCodes([21830, 21697]),
].join("")

const productIdentityPrompt = [
  textFromCodes([35831, 25552, 20379]),
  textFromCodes([21654, 21857, 31867, 21830, 21697]),
  textFromCodes([30340, 20135, 21697, 32534, 21495]),
  textFromCodes([25110, 21517, 31216]),
  textFromCodes([20197, 20415, 26597, 35810]),
  textFromCodes([30456, 20851, 19994, 21153, 25351, 26631]),
].join("")

describe("legacy category boundary runtime events", () => {
  it("downgrades legacy product data failures without explicit boundary marker", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: coffeeCategoryTask,
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: "worker.data",
              title: ["Data", "diagnosis"].join(" "),
              objective: "category-level data diagnosis",
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
            {
              id: "service",
              role: "service",
              worker_id: "worker.service",
              title: ["Service", "support"].join(" "),
              objective: "category-level service boundary",
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
          ],
        },
      },
      {
        type: "skill_result",
        request_id: "skill-1",
        skill_id: "query_product_metrics",
        role: "data",
        status: "validation_error",
        summary: productIdentityPrompt,
      },
      {
        type: "skill_result",
        request_id: "skill-2",
        skill_id: "service_query_product",
        role: "service",
        status: "failed",
        summary: "Product context awaits explicit product selection.",
      },
      {
        type: "task_node_failed",
        node_id: "data",
        reason: "data role should provide at least two findings",
      },
      {
        type: "task_node_failed",
        node_id: "service",
        reason: "service role output lacks service-specific content",
      },
      { type: "task_graph_finished", graph_id: "graph-1" },
    ]

    expect(
      getInlineProcessStage({
        hasRuntime: true,
        isStreaming: false,
        runtimeEvents: events,
      }, t)
    ).toBe("finished")
    expect(getInlineAgents(events, false, t).find((agent) => agent.nodeId === "data")?.state).not.toBe("failed")
    expect(getInlineAgents(events, false, t).find((agent) => agent.nodeId === "service")?.state).not.toBe("failed")
    expect(getInlineAgents(events, false, t).find((agent) => agent.nodeId === "data")?.state).toBe("boundary")
    expect(getInlineAgents(events, false, t).find((agent) => agent.nodeId === "service")?.state).toBe("boundary")
    expect(getRuntimeNodeStates(events).get("data")?.state).not.toBe("failed")
    expect(getRuntimeNodeStates(events).get("service")?.state).not.toBe("failed")
  })

  it("renders legacy non-blocking node and workspace task failures as boundary review", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: coffeeCategoryTask,
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: "worker.data",
              title: ["Data", "diagnosis"].join(" "),
              objective: ["category-level", "data", "diagnosis"].join(" "),
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
          ],
        },
      },
      {
        type: "skill_result",
        request_id: "skill-1",
        skill_id: "query_product_metrics",
        role: "data",
        status: "validation_error",
        summary: productIdentityPrompt,
      },
      {
        type: "task_node_failed",
        node_id: "data",
        reason: "data role should provide at least two findings",
      },
      {
        type: "workspace_run_task_updated",
        id: "run-task-1",
        ownerRole: "data",
        status: "failed",
        metadata: {
          report: {
            roleOutput: {
              evidence: [categoryBoundaryMarker()],
            },
          },
        },
      },
      {
        type: "workspace_run_failed",
        id: "run-1",
        status: "failed",
        summary: "run stopped after boundary review",
      },
    ]

    const timeline = events
      .map((event) =>
        formatTimelineEvent(
          event,
          [
            {
              nodeId: "data",
              name: ["d", "ata"].join(""),
              role: "data",
              summary: "",
              icon: BarChart3,
              tone: "",
              objective: "",
              dependsOn: [],
            },
          ],
          events,
          t
        )
      )
      .filter(Boolean)

    expect(timeline.some((item) => item?.label === "node boundary")).toBe(true)
    expect(timeline.some((item) => item?.label === "workspace boundary")).toBe(true)
    expect(getWorkspacePersistenceCard(events, t)?.state).toBe("boundary")
    expect(hasBlockingWorkspaceFailure(events)).toBe(false)
    expect(getRuntimeNodeStates(events).get("delivery")?.state).toBe("boundary")
  })

  it("downgrades category-level coordination and role permission skill gaps", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: coffeeCategoryTask,
          nodes: [
            {
              id: "engineering",
              role: "engineering",
              worker_id: "worker.engineering",
              title: ["Engineer", "ing"].join(""),
              objective: "category-level engineering boundary",
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
          ],
        },
      },
      {
        type: "skill_result",
        request_id: "skill-1",
        skill_id: "coordination_create_task",
        role: "engineering",
        status: "validation_error",
        error: "skill 'coordination_create_task' is not allowed for role 'engineering'",
      },
      {
        type: "skill_result",
        request_id: "skill-2",
        skill_id: "coordination_create_task",
        role: "engineering",
        status: "validation_error",
        error: "invalid parameter 'priority': expected integer",
      },
      { type: "task_graph_finished", graph_id: "graph-1" },
    ]

    expect(getInlineAgents(events, false, t).find((agent) => agent.nodeId === "engineering")?.state).toBe("boundary")
    expect(getRuntimeNodeStates(events).get("engineering")?.state).toBe("boundary")
    expect(
      getInlineProcessStage({
        hasRuntime: true,
        isStreaming: false,
        runtimeEvents: events,
      }, t)
    ).not.toBe("failed")
  })

  it("treats coffee category quality gaps as boundary without an explicit product", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-coffee",
          root_task: textFromCodes([35831, 20998, 26512, 21654, 21857, 31867, 21830, 21697]),
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: "worker.data",
              title: ["D", "ata"].join(""),
              objective: "coffee category data diagnosis",
              input_refs: [],
              acceptance: [],
              max_attempts: 1,
            },
          ],
        },
      },
      {
        type: "task_node_reported",
        node_id: "data",
        worker_id: "worker.data",
        report: {
          graph_id: "graph-coffee",
          node_id: "data",
          task_id: "task-data",
          worker_id: "worker.data",
          role: "data",
          content: textFromCodes([21654, 21857, 31867, 30446, 25968, 25454, 36793, 30028, 35828, 26126]),
          roleOutput: {
            summary: textFromCodes([21654, 21857, 31867, 21830, 21697, 32570, 23569, 30495, 23454, 26333, 20809, 12289, 28857, 20987, 12289, 21152, 36141, 21644, 20184, 27454, 25351, 26631, 12290]),
            findings: [textFromCodes([24403, 21069, 27809, 26377, 30495, 23454, 28431, 26007, 25968, 25454, 12290])],
            recommendations: [],
            evidence: [textFromCodes([29992, 25143, 26412, 36718, 27809, 26377, 25163, 21160, 36873, 25321, 39044, 35774, 21830, 21697, 12290])],
            risks: [],
            openQuestions: [textFromCodes([35831, 25552, 20379, 21654, 21857, 31867, 21830, 21697, 30340, 20855, 20307, 25351, 26631, 26469, 28304, 12290])],
          },
          evidence: [],
          risks: [],
          open_questions: [],
          status: "failed",
        },
      },
      {
        type: "task_node_failed",
        node_id: "data",
        reason: "data role should provide at least two findings",
      },
      { type: "task_graph_finished", graph_id: "graph-coffee" },
    ]

    expect(hasBlockingTaskFailure(events)).toBe(false)
    expect(getInlineAgents(events, false, t).find((agent) => agent.nodeId === "data")?.state).toBe("boundary")
    expect(getRuntimeNodeStates(events).get("data")?.state).toBe("boundary")
  })

})

function textFromCodes(codes: number[]) { return String.fromCodePoint(...codes) }

function categoryBoundaryMarker() { return "category_level_boundary" }
