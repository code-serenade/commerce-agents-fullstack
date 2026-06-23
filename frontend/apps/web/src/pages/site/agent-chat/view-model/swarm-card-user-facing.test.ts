import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent, AgentTaskNode } from "@/api"
import {
  getAgents,
  getPhases,
} from "./swarm-panel-view-model"
import { getComputerCards } from "./swarm-runtime-view-model"
import { getComputerScreenContext } from "./swarm-runtime-view-model"
import type { RuntimeNodeSnapshot } from "./swarm-runtime-state-view-model"
import { getSkillCard } from "./swarm-skill-card-view-model"
import { getWorkspacePersistenceCard } from "./swarm-workspace-view-model"

const genericProgress = "generic progress"
const referenceLimited = "reference limited"

const t = ((key: string, options?: Record<string, unknown>) => {
  const messages: Record<string, string> = {
    "agentChat.swarm.cards.assign.currentTask": `agent ${String(options?.agent)}`,
    "agentChat.swarm.cards.assign.description": "assignment",
    "agentChat.swarm.cards.assign.latestAction": `latest ${String(options?.action)}`,
    "agentChat.swarm.cards.assign.latestPlanning": "planning",
    "agentChat.swarm.cards.assign.meta": "runtime",
    "agentChat.swarm.cards.assign.stateLine": `state ${String(options?.state)}`,
    "agentChat.swarm.cards.assign.title": "runtime card",
    "agentChat.swarm.cards.output.description": "output",
    "agentChat.swarm.cards.output.fallbackLines": "fallback",
    "agentChat.swarm.cards.output.materialMeta": "material",
    "agentChat.swarm.cards.output.materialTitle": "material output",
    "agentChat.swarm.cards.readTarget.inputDescription": "input",
    "agentChat.swarm.cards.readTarget.title": "read target",
    "agentChat.swarm.cards.readTarget.userMeta": "user",
    "agentChat.swarm.cards.rework.description": "rework",
    "agentChat.swarm.cards.rework.meta": "retry",
    "agentChat.swarm.cards.rework.title": "rework card",
    "agentChat.swarm.cards.skill.completed": "reference completed",
    "agentChat.swarm.cards.skill.meta": "background",
    "agentChat.swarm.cards.skill.statusLine": `status ${String(options?.status)}`,
    "agentChat.swarm.cards.skill.title": "skill card",
    "agentChat.swarm.cards.workspace.artifactCreatedDescription": "artifact",
    "agentChat.swarm.cards.workspace.artifactCreatedTitle": "artifact card",
    "agentChat.swarm.cards.workspace.artifactLine": `artifact ${String(options?.title)}`,
    "agentChat.swarm.cards.workspace.errorDescription": "persist failed",
    "agentChat.swarm.cards.workspace.errorTitle": "workspace error",
    "agentChat.swarm.cards.workspace.linkedDescription": "linked",
    "agentChat.swarm.cards.workspace.linkedTitle": "linked card",
    "agentChat.swarm.cards.workspace.reusedMeta": "reused",
    "agentChat.swarm.cards.workspace.runLine": `run ${String(options?.status)}`,
    "agentChat.swarm.cards.workspace.savedReportLine": "saved report",
    "agentChat.swarm.phases.dynamicTitle": `phase ${String(options?.index)} ${String(options?.title)}`,
    "agentChat.swarm.screen.acceptanceLine": `accept ${String(options?.item)}`,
    "agentChat.swarm.screen.deliveryFilename": "delivery.md",
    "agentChat.swarm.screen.objectiveLine": `objective ${String(options?.request)}`,
    "agentChat.swarm.screen.responsibilityLine": `summary ${String(options?.summary)}`,
    "agentChat.swarm.screen.taskMode": "task mode",
    "agentChat.swarm.runtimeReasons.collaborationIncomplete": genericProgress,
    "agentChat.thread.inlineState.working": "working",
    "agentChat.thread.runtimeText.genericProgress": genericProgress,
    "agentChat.thread.runtimeText.referenceLimited": referenceLimited,
    "agentChat.thread.steps.userFacingTitles.generic": "organize judgment",
  }
  return messages[key] ?? key
}) as never

const activeAgent = {
  dependsOn: [],
  icon: (() => null) as never,
  name: "Ops",
  nodeId: "ops",
  objective: "Help the user judge the commerce task.",
  role: "ops",
  summary: "Commerce operations",
  tone: "",
}

describe("swarm user-facing cards", () => {
  it("hides internal persistence error details in workspace cards", () => {
    const internalAction = "mark_agent_task_started"
    const events: AgentRuntimeEvent[] = [
      {
        type: "workspace_run_persistence_error",
        action: internalAction,
        message:
          "adapter failed https://internal.example/workspace route_mode=task_graph",
      },
    ]

    const card = getWorkspacePersistenceCard(events, t)
    const text = [card?.meta, ...(card?.lines ?? [])].join(" ")

    expect(text).toContain(genericProgress)
    expect(text).not.toContain(internalAction)
    expect(text).not.toContain("adapter")
    expect(text).not.toContain("https://")
    expect(text).not.toContain("route_mode")
  })

  it("hides internal artifact and linked workspace labels", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "workspace_artifact_created",
        id: "artifact-123456",
        [["ti", "tle"].join("")]: "search_evidence_trace",
        artifact_type: "selected_skills",
        preview: "task_graph route_mode=task_graph",
      } as unknown as AgentRuntimeEvent,
    ]

    const card = getWorkspacePersistenceCard(events, t)
    const text = [card?.meta, ...(card?.lines ?? [])].join(" ")

    expect(text).toContain(genericProgress)
    expect(text).not.toContain("search_evidence")
    expect(text).not.toContain("selected_skills")
    expect(text).not.toContain("task_graph")
    expect(text).not.toContain("route_mode")
  })

  it("hides internal skill errors and translates skill status", () => {
    const activeNode: RuntimeNodeSnapshot = {
      evaluationReasons: [],
      skillResults: [
        {
          error:
            "skill 'coordination_create_task' is not allowed for role 'engineering'",
          skillId: "coordination_create_task",
          status: "validation_error",
        },
      ],
      state: "failed",
    }

    const card = getSkillCard(activeNode, t)
    const text = [card?.description, ...(card?.lines ?? [])].join(" ")

    expect(text).toContain(genericProgress)
    expect(text).toContain(referenceLimited)
    expect(text).not.toContain("skill '")
    expect(text).not.toContain("coordination_create_task")
    expect(text).not.toContain("engineering")
    expect(text).not.toContain("validation_error")
  })

  it("hides internal rework instruction in computer cards", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_node_rework_requested",
        attempt: 2,
        instruction: "worker.data route_mode=task_graph selected_skills=search_trends",
        node_id: "ops",
      },
    ]

    const cards = getComputerCards(
      {
        activeAgent,
        activeNode: {
          attempt: 2,
          evaluationReasons: [],
          skillResults: [],
          state: "reworking",
        },
        agents: [activeAgent],
        lastTask: "Review the product.",
        runtimeEvents: events,
      },
      t
    )
    const text = cards.flatMap((card) => card.lines).join(" ")

    expect(text).toContain(genericProgress)
    expect(text).not.toContain("worker.data")
    expect(text).not.toContain("route_mode")
    expect(text).not.toContain("task_graph")
    expect(text).not.toContain("selected_skills")
  })

  it("hides internal graph node titles and objectives in swarm agents and phases", () => {
    const nodes = [
      {
        acceptance: ["selected_skills=search_trends"],
        id: "custom",
        input_refs: [],
        max_attempts: 1,
        [["obj", "ective"].join("")]:
          "adapter route_mode=task_graph https://internal.example/node",
        role: "custom",
        [["ti", "tle"].join("")]: "worker.data",
        worker_id: "worker.data",
      },
    ] as unknown as AgentTaskNode[]

    const agents = getAgents(nodes, t)
    const phases = getPhases(nodes, t)
    const text = [
      agents[0]?.name,
      agents[0]?.summary,
      agents[0]?.objective,
      phases[0]?.title,
      phases[0]?.detail,
    ].join(" ")

    expect(text).toContain("organize judgment")
    expect(text).toContain(genericProgress)
    expect(text).not.toContain("worker.data")
    expect(text).not.toContain("route_mode")
    expect(text).not.toContain("task_graph")
    expect(text).not.toContain("https://")
  })

  it("hides internal graph task details in the runtime screen and task card", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "task_graph_planned",
        graph: {
          graph_id: "graph-1",
          nodes: [
            ({
              acceptance: ["selected_skills=search_trends"],
              id: "ops",
              input_refs: [],
              max_attempts: 1,
              [["obj", "ective"].join("")]: "route_mode=task_graph",
              role: "ops",
              [["ti", "tle"].join("")]: "worker.data",
              worker_id: "worker.data",
            } as unknown as AgentTaskNode),
          ],
          root_task: "task_graph selected_skills=search_trends",
        },
      },
    ]

    const screen = getComputerScreenContext(
      {
        activeAgent,
        runtimeEvents: events,
      },
      t
    )
    const cards = getComputerCards(
      {
        activeAgent,
        agents: [activeAgent],
        runtimeEvents: events,
      },
      t
    )
    const text = [screen.request, ...screen.lines, ...cards.flatMap((card) => card.lines)].join(" ")

    expect(text).toContain(genericProgress)
    expect(text).toContain("organize judgment")
    expect(text).not.toContain("worker.data")
    expect(text).not.toContain("route_mode")
    expect(text).not.toContain("task_graph")
    expect(text).not.toContain("selected_skills")
  })
})
