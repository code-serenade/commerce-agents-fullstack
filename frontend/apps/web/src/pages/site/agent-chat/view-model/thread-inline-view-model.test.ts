import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getInlineArtifactsFromRuntime } from "./thread-artifacts-view-model"
import { isChatOnlyRuntime } from "./thread-inline-view-model"

const t = ((key: string, params?: Record<string, unknown>) => {
  if (key === "agentChat.thread.artifacts.searchEvidence") {
    return "search evidence"
  }
  if (key === "agentChat.thread.artifacts.backendCreated") {
    return `${String(params?.type)}:${String(params?.status)}`
  }
  return key
}) as never

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
