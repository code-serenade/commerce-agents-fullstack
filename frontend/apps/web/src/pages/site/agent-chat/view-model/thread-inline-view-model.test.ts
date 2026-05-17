import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getInlineArtifactsFromRuntime } from "./thread-artifacts-view-model"
import {
  getInlineAgents,
  getInlineProcessStage,
  isChatOnlyRuntime,
} from "./thread-inline-view-model"
import { getRuntimeNodeStates } from "./swarm-runtime-state-view-model"

const t = ((key: string, params?: Record<string, unknown>) => {
  if (key === "agentChat.thread.artifacts.searchEvidence") {
    return "search evidence"
  }
  if (key === "agentChat.thread.artifacts.backendCreated") {
    return `${String(params?.type)}:${String(params?.status)}`
  }
  if (key === "agentChat.thread.processStage.finished") {
    return "finished"
  }
  if (key === "agentChat.thread.processStage.failed") {
    return "failed"
  }
  if (key === "agentChat.thread.inlineState.done") {
    return "done"
  }
  if (key === "agentChat.thread.agents.data.name") {
    return "data"
  }
  if (key === "agentChat.thread.agents.data.summary") {
    return "data summary"
  }
  if (key === "agentChat.thread.agents.delivery.name") {
    return "delivery"
  }
  if (key === "agentChat.thread.agents.delivery.summary") {
    return "delivery summary"
  }
  return key
}) as never

const categoryBoundary = "category_level_boundary"
const dataNodeTitle = ["Data", " diagnosis"].join("")

describe("isChatOnlyRuntime", () => {
  it("treats chat_bypass route as direct chat without swarm UI", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "route_decided",
        route_mode: "chat_bypass",
        primary_runtime_role: "chat",
        support_runtime_roles: [],
      },
    ]

    expect(isChatOnlyRuntime(events)).toBe(true)
  })

  it("keeps task graph route visible for multi-role work", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "route_decided",
        route_mode: "task_graph",
        primary_runtime_role: "ops",
        support_runtime_roles: ["data", "creative"],
      },
    ]

    expect(isChatOnlyRuntime(events)).toBe(false)
  })

  it("does not hide task graph UI only because the primary role is chat", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "route_decided",
        route_mode: "task_graph",
        primary_runtime_role: "chat",
        support_runtime_roles: [],
      },
    ]

    expect(isChatOnlyRuntime(events)).toBe(false)
  })
})

describe("getInlineArtifactsFromRuntime", () => {
  it("uses only backend-created artifacts instead of demo artifact specs", () => {
    const finalReportTitle = ["Agent", " final report"].join("")
    const finalReportType = ["final", "_report"].join("")
    const finalReportBody = ["Final", " answer body"].join("")
    const searchEvidenceTitle = ["Live", " search evidence"].join("")
    const searchEvidenceType = ["search", "_evidence"].join("")
    const searchEvidenceStatus = ["search", " evidence"].join("")
    const events: AgentRuntimeEvent[] = [
      {
        type: "workspace_artifact_created",
        id: "artifact-1",
        title: finalReportTitle,
        status: "active",
        artifactType: finalReportType,
        content: finalReportBody,
        preview: finalReportBody,
      },
      {
        type: "workspace_artifact_created",
        id: "artifact-2",
        title: searchEvidenceTitle,
        status: "active",
        artifact_type: searchEvidenceType,
        format: "json",
      },
    ]

    const artifacts = getInlineArtifactsFromRuntime(events, t)

    expect(artifacts).toEqual([
      {
        filename: finalReportTitle,
        meta: finalReportType,
        nodeId: "delivery",
        status: finalReportBody,
        state: "done",
      },
      {
        filename: searchEvidenceTitle,
        meta: searchEvidenceType,
        nodeId: "delivery",
        status: searchEvidenceStatus,
        state: "done",
      },
    ])
  })

  it("returns an empty list when backend has not created artifacts", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "skill_result",
        request_id: "skill-1",
        skill_id: "design_material_spec",
        role: "design",
        status: "success",
      },
    ]

    expect(getInlineArtifactsFromRuntime(events, t)).toEqual([])
  })
})

describe("thread process boundary states", () => {
  it("treats category-level product metric gaps as data boundaries instead of failed progress", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: "coffee category analysis",
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: "worker.data",
              title: dataNodeTitle,
              objective: "category-level data diagnosis",
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
        report: {
          graph_id: "graph-1",
          node_id: "data",
          task_id: "task-1",
          worker_id: "worker.data",
          role: "data",
          content: categoryBoundary,
          role_output: {
            summary: categoryBoundary,
            findings: ["no fabricated funnel data"],
            recommendations: ["produce category-level framework first"],
            evidence: [categoryBoundary],
            risks: [categoryBoundary],
            open_questions: [],
          },
          evidence: [categoryBoundary],
          risks: [categoryBoundary],
          open_questions: [],
          status: "completed",
        },
      },
      {
        type: "skill_result",
        request_id: "skill-1",
        skill_id: "query_product_metrics",
        role: "data",
        status: "validation_error",
        summary: categoryBoundary,
      },
      {
        type: "task_node_failed",
        node_id: "data",
        reason: categoryBoundary,
      },
      { type: "task_graph_finished", graph_id: "graph-1" },
    ]

    expect(
      getInlineProcessStage({ hasRuntime: true, isStreaming: false, runtimeEvents: events }, t)
    ).toBe("finished")
    expect(getInlineAgents(events, false, t).find((agent) => agent.nodeId === "data")?.state).toBe("boundary")
  })

  it("reads category boundary marker from skill output when summary is generic", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: "coffee category analysis",
          nodes: [
            {
              id: "data",
              role: "data",
              worker_id: "worker.data",
              title: dataNodeTitle,
              objective: "category-level data diagnosis",
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
        summary: ["Product metric query", "awaits real input"].join(" "),
        output: {
          boundary_marker: categoryBoundary,
          boundary_policy: "category_level_without_selected_product",
        },
      },
      {
        type: "task_node_failed",
        node_id: "data",
        reason: categoryBoundary,
      },
      { type: "task_graph_finished", graph_id: "graph-1" },
    ]

    expect(
      getInlineProcessStage({ hasRuntime: true, isStreaming: false, runtimeEvents: events }, t)
    ).toBe("finished")
    expect(getInlineAgents(events, false, t).find((agent) => agent.nodeId === "data")?.state).not.toBe("failed")
    expect(getRuntimeNodeStates(events).get("data")?.state).not.toBe("failed")
  })

  it("keeps service product context gaps non-blocking for category answers", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          root_task: "coffee category analysis",
          nodes: [
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
        skill_id: "service_query_product",
        role: "service",
        status: "deferred",
        summary: ["Product context", "awaits explicit selection"].join(" "),
        output: {
          boundary_marker: categoryBoundary,
          adapter_status: "context_required",
        },
      },
      {
        type: "task_node_failed",
        node_id: "service",
        reason: categoryBoundary,
      },
      { type: "task_graph_finished", graph_id: "graph-1" },
    ]

    expect(getInlineAgents(events, false, t).find((agent) => agent.nodeId === "service")?.state).not.toBe("failed")
    expect(getRuntimeNodeStates(events).get("service")?.state).not.toBe("failed")
  })

})
