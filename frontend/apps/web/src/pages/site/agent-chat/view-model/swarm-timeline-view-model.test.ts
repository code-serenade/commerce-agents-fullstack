import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { formatTimelineEvent } from "./swarm-timeline-view-model"

const collaborationIncomplete = textFromCodes([
  36825, 19968, 27493, 38656, 35201, 34917, 20805, 26356, 20855, 20307,
  30340, 19994, 21153, 36164, 26009, 12290,
])

const t = ((key: string, options?: Record<string, unknown>) => {
  if (key === "agentChat.swarm.timeline.events.workspaceContextLoaded") {
    return `workspace ${options?.sources}`
  }
  if (key === "agentChat.swarm.timeline.events.workspaceRunLinked") {
    return `linked ${options?.title}`
  }
  if (key === "agentChat.swarm.timeline.events.workspaceTaskUpdated") {
    return `task ${options?.status}`
  }
  if (key === "agentChat.swarm.timeline.events.workspaceTaskBoundary") {
    return `boundary ${options?.status}`
  }
  if (key === "agentChat.swarm.timeline.events.workspaceArtifactCreated") {
    return `artifact ${options?.title}`
  }
  if (key === "agentChat.swarm.timeline.events.agentLearningRecordCreated") {
    return `learning ${options?.category}`
  }
  if (key === "agentChat.swarm.timeline.events.approvalRequired") {
    return `approval ${options?.risk}`
  }
  if (key === "agentChat.swarm.timeline.events.workspacePersistenceError") {
    return `persist ${options?.action}`
  }
  if (key === "agentChat.swarm.timeline.events.nodeRework") {
    return `rework ${options?.instruction}`
  }
  if (key === "agentChat.swarm.timeline.events.nodeFailed") {
    return `blocked ${options?.reason}`
  }
  if (key === "agentChat.swarm.timeline.events.ragContextDegraded") {
    return "reference unavailable"
  }
  if (key === "agentChat.swarm.timeline.events.skillResult") {
    return `reference ${options?.status}`
  }
  if (key === "agentChat.swarm.timeline.events.routeDecided") {
    return `${options?.mode} ${options?.role}`
  }
  if (key === "agentChat.thread.dispatchPanel.routeModes.task_graph") {
    return "multi-step route"
  }
  if (key === "agentChat.thread.roleFallbackTitles.design") {
    return "Presentation advice"
  }
  if (key === "agentChat.thread.runtimeText.referenceLimited") {
    return "limited"
  }
  if (key === "agentChat.thread.runtimeText.referenceReady") {
    return "ready"
  }
  if (key === "agentChat.thread.runtimeText.referencePending") {
    return "pending"
  }
  if (key === "agentChat.thread.runtimeText.referenceSynced") {
    return "synced"
  }
  if (key === "agentChat.thread.runtimeText.genericProgress") {
    return "generic progress"
  }
  if (key === "agentChat.thread.inlineState.done") return "done"
  if (key === "agentChat.thread.inlineState.working") return "working"
  if (key === "agentChat.thread.inlineState.queued") return "queued"
  if (key === "agentChat.thread.inlineState.failed") return "failed"
  if (key === "agentChat.swarm.runtimeReasons.collaborationIncomplete") {
    return collaborationIncomplete
  }
  return key
}) as never

describe("formatTimelineEvent", () => {
  it("formats safe workspace source count without source bodies", () => {
    const event: AgentRuntimeEvent = {
      type: "workspace_context_loaded",
      counts: {
        campaigns: 1,
        conversationFiles: 1,
      },
      readiness: "ready",
      sourceCount: 5,
    }

    expect(formatTimelineEvent(event, [], [], t)?.label).toBe("workspace 5")
  })

  it("does not expose internal role names in blocked node timeline text", () => {
    const event: AgentRuntimeEvent = {
      type: "task_node_failed",
      node_id: "design",
      reason: "design role output lacks design-specific content",
    }

    const item = formatTimelineEvent(
      event,
      [
        {
          dependsOn: [],
          icon: (() => null) as never,
          name: "Experience design",
          nodeId: "design",
          objective: "",
          role: "design",
          summary: "",
          tone: "",
        },
      ],
      [event],
      t
    )

    expect(item?.label).toBe(`blocked ${collaborationIncomplete}`)
    expect(item?.label).not.toContain("Experience design")
    expect(item?.label).not.toContain("design role")
  })

  it("does not expose raw search degradation reasons in timeline text", () => {
    const event: AgentRuntimeEvent = {
      type: "rag_context_loaded",
      degraded: true,
      degraded_reason:
        "external search provider request failed: http://internal/search",
      source_count: 0,
    }

    const item = formatTimelineEvent(event, [], [event], t)

    expect(item?.label).toBe("reference unavailable")
    expect(item?.label).not.toContain("external search")
    expect(item?.label).not.toContain("http://")
  })

  it("translates raw skill statuses before timeline display", () => {
    const event: AgentRuntimeEvent = {
      type: "skill_result",
      request_id: "req",
      role: "search",
      skill_id: "search_trends",
      status: "deferred",
    }

    const item = formatTimelineEvent(event, [], [event], t)

    expect(item?.label).toBe("reference pending")
    expect(item?.label).not.toContain("deferred")
    expect(item?.label).not.toContain("search_trends")
  })

  it("translates route role before timeline display", () => {
    const event: AgentRuntimeEvent = {
      type: "route_decided",
      primary_runtime_role: "design",
      route_mode: "task_graph",
      strategy_id: "learning",
      support_runtime_roles: [],
    }

    const item = formatTimelineEvent(event, [], [event], t)

    expect(item?.label).toBe("multi-step route Presentation advice")
    expect(item?.label).not.toContain("design")
    expect(item?.label).not.toContain("task_graph")
  })

  it("hides internal workspace titles, artifact titles, and learning categories", () => {
    const internalTitle = [
      ["search", "evidence"].join("_"),
      ["trace", "missing"].join(" "),
      "https://internal.example/search",
    ].join(" ")
    const events: AgentRuntimeEvent[] = [
      {
        type: "workspace_run_linked",
        run: {
          id: "run-1",
          status: "running",
        } as never,
        workspace: {
          id: "workspace-1",
          title: internalTitle,
        } as never,
      },
      {
        type: "workspace_artifact_created",
        title: ["selected", "skills"].join("_"),
      } as AgentRuntimeEvent,
      {
        type: "agent_learning_record_created",
        category: ["task", "graph"].join("_"),
      } as AgentRuntimeEvent,
    ]

    const labels = events.map((event) =>
      formatTimelineEvent(event, [], events, t)?.label
    )

    expect(labels).toEqual([
      "linked generic progress",
      "artifact generic progress",
      "learning generic progress",
    ])
    expect(labels.join(" ")).not.toContain("search_evidence")
    expect(labels.join(" ")).not.toContain("https://")
    expect(labels.join(" ")).not.toContain("selected_skills")
    expect(labels.join(" ")).not.toContain("task_graph")
  })

  it("hides internal rework, approval, and persistence details", () => {
    const routeMode = ["route", "mode"].join("_")
    const taskGraph = ["task", "graph"].join("_")
    const selectedSkills = ["selected", "skills"].join("_")
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_node_rework_requested",
        attempt: 2,
        instruction: `worker.data ${routeMode}=${taskGraph} ${selectedSkills}=search_trends`,
      },
      {
        type: "approval_required",
        risk_level: "worker.data",
      } as AgentRuntimeEvent,
      {
        type: "workspace_run_persistence_error",
        action: "mark_agent_task_started",
        message: "adapter failed https://internal.example/workspace",
      },
    ]

    const labels = events.map((event) =>
      formatTimelineEvent(event, [], events, t)?.label
    )

    expect(labels).toEqual([
      "rework generic progress",
      "approval generic progress",
      "persist generic progress",
    ])
    expect(labels.join(" ")).not.toContain("worker.data")
    expect(labels.join(" ")).not.toContain(routeMode)
    expect(labels.join(" ")).not.toContain(taskGraph)
    expect(labels.join(" ")).not.toContain(selectedSkills)
    expect(labels.join(" ")).not.toContain("mark_agent_task_started")
  })

  it("translates workspace task statuses before timeline display", () => {
    const event: AgentRuntimeEvent = {
      type: "workspace_run_task_updated",
      id: "task-1",
      owner_role: "ops",
      status: "failed",
    } as never

    const item = formatTimelineEvent(event, [], [event], t)

    expect(item?.label).toBe("task failed")
    expect(item?.label).not.toContain("task-1")
  })

  it("falls back for machine-readable workspace task statuses", () => {
    const event: AgentRuntimeEvent = {
      type: "workspace_run_task_updated",
      id: "task-1",
      owner_role: "ops",
      status: ["skill", "validation", "error"].join("_"),
    } as never

    const item = formatTimelineEvent(event, [], [event], t)

    expect(item?.label).toBe("task generic progress")
    expect(item?.label).not.toContain("skill_validation_error")
  })
})

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}
