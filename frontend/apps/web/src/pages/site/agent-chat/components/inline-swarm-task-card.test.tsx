import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import {
  node,
  reported,
} from "../view-model/agent-process-facts-test-helpers"
import { InlineSwarmTaskCard } from "./inline-swarm-task-card"

const task = "diagnose campaign material"
const dataRole = "data"
const opsRole = "ops"
const searchRole = "search"
const searchClaimText = "competitor discount"
const searchSourceTitle = "market source"

describe("InlineSwarmTaskCard", () => {
  it("shows audit facts together with the agent grid for multi-agent runs", async () => {
    const user = userEvent.setup()
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <InlineSwarmTaskCard
          isStreaming={false}
          messageId="message-1"
          onCloseProcessPanel={vi.fn()}
          onOpenProcessPanel={vi.fn()}
          processPanelOpen={false}
          runtimeEvents={createMultiAgentAuditEvents()}
          showProcessPanelAction={false}
          task={task}
        />
      </AgentChatTestI18nProvider>
    )

    await user.click(screen.getByTitle(i18n.t("agentChat.thread.expandProcess")))

    expect(
      screen.getAllByText(i18n.t("agentChat.thread.agents.data.name")).length
    ).toBeGreaterThan(0)
    expect(
      screen.getByText(i18n.t("agentChat.thread.dispatchFacts.roleAudit.label"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.dispatchFacts.searchAudit.label"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.dispatchFacts.materialAudit.label"))
    ).toBeInTheDocument()
  })

  it("shows completed badge after final output instead of in-progress wording", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <InlineSwarmTaskCard
          isStreaming={false}
          messageId="message-1"
          onCloseProcessPanel={vi.fn()}
          onOpenProcessPanel={vi.fn()}
          processPanelOpen={false}
          runtimeEvents={[
            ...createMultiAgentAuditEvents(),
            {
              type: "output",
              content: "final answer",
            },
          ]}
          showProcessPanelAction={false}
          task={task}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(i18n.t("agentChat.thread.processKinds.completed.badge"))
    ).toBeInTheDocument()
    expect(
      screen.queryByText(i18n.t("agentChat.thread.processKinds.multi_agent.badge"))
    ).not.toBeInTheDocument()
  })
})

function createMultiAgentAuditEvents(): AgentRuntimeEvent[] {
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
        root_task: task,
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
          sources: [{ title: searchSourceTitle }],
          usable_claims: [{ text: searchClaimText }],
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
