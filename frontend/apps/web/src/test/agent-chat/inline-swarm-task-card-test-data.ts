import type { AgentRuntimeEvent } from "@/api"
import {
  node,
  reported,
} from "@/pages/site/agent-chat/view-model/agent-process-facts-test-helpers"

export const inlineSwarmTaskCardTestTask = "diagnose campaign material"

const dataRole = "data"
const opsRole = "ops"
const searchRole = "search"

export function createQuickDispatchEvents(): AgentRuntimeEvent[] {
  return [
    {
      type: "route_decided",
      primary_runtime_role: "chat",
      route_mode: "chat_bypass",
      support_runtime_roles: [],
    },
    {
      content: "quick answer",
      type: "output",
    },
  ]
}

export function createMultiAgentAuditEvents(): AgentRuntimeEvent[] {
  return [
    {
      type: "dispatch_planned",
      difficulty: "complex",
      planned_steps: [],
      primary_runtime_role: dataRole,
      route_mode: "task_graph",
      selected_skills: ["search_competitor"],
      student_visible_explanation: "",
      support_runtime_roles: [opsRole, searchRole],
    },
    {
      type: "task_graph_planned",
      graph: {
        graph_id: "graph-1",
        root_task: inlineSwarmTaskCardTestTask,
        nodes: [
          node(dataRole, dataRole),
          node(opsRole, opsRole),
          node(searchRole, searchRole),
        ],
      },
    },
    reported(dataRole, dataRole),
    reported(opsRole, opsRole),
    {
      type: "skill_result",
      output: {
        evidence_packet: {
          confidence: 0.72,
          gaps: [],
          search_strength: "standard",
          sources: [],
          usable_claims: [],
        },
      },
      request_id: "request-1",
      role: searchRole,
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
        {
          id: "file-a",
          refKey: "conversation_file:file-a",
          text: "readable text",
        },
      ],
      readiness: "ready",
    },
  ]
}

export function createStreamingGraphEvents(): AgentRuntimeEvent[] {
  return [
    {
      type: "task_graph_planned",
      graph: {
        graph_id: "graph-streaming",
        root_task: inlineSwarmTaskCardTestTask,
        nodes: [
          node(dataRole, dataRole),
          node(opsRole, opsRole),
        ],
      },
    },
    {
      attempt: 1,
      graph_id: "graph-streaming",
      node_id: dataRole,
      type: "task_node_started",
      worker_id: `worker.${dataRole}`,
    },
  ]
}
