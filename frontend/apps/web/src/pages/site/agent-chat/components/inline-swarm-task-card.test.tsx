import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import {
  createMultiAgentAuditEvents,
  createQuickDispatchEvents,
  createStreamingGraphEvents,
  inlineSwarmTaskCardTestTask,
} from "@/test/agent-chat/inline-swarm-task-card-test-data"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { InlineSwarmTaskCard } from "./inline-swarm-task-card"

const task = inlineSwarmTaskCardTestTask

describe("InlineSwarmTaskCard", () => {
  it("shows the active role portrait for quick dispatch process cards", async () => {
    const user = userEvent.setup()
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <InlineSwarmTaskCard
          isStreaming={false}
          messageId="message-quick"
          onCloseProcessPanel={vi.fn()}
          onOpenProcessPanel={vi.fn()}
          processPanelOpen={false}
          runtimeEvents={createQuickDispatchEvents()}
          showProcessPanelAction={false}
          task={task}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(
        i18n.t("agentChat.thread.roleClusterActivity.participants")
      )
    ).toBeInTheDocument()
    expect(
      document.querySelector("[data-agent-role-participant='ops']")
    ).not.toBeNull()
    expect(
      document
        .querySelector("[data-process-message-id='message-quick']")
        ?.querySelector("[data-agent-process-card-frame='true']")
    ).toHaveClass("w-full")
    expect(
      document.querySelector("[data-agent-portrait-asset='ops-portrait']")
    ).not.toBeNull()

    await user.click(
      screen.getByTitle(i18n.t("agentChat.thread.expandProcess"))
    )

    expect(
      screen.getByText(i18n.t("agentChat.thread.roleClusterActivity.title"))
    ).toBeInTheDocument()
    expect(
      document.querySelector("[data-agent-process-expanded-content='true']")
    ).toHaveClass("overflow-y-auto")
    expect(
      document.querySelector("[data-agent-role-card='ops']")
    ).not.toBeNull()
    expect(
      document.querySelector("[data-agent-portrait-asset='ops-portrait']")
    ).toHaveClass("object-contain")
    expect(document.querySelector("[data-agent-role-card='data']")).toBeNull()
  })

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

    expect(
      screen.getByText(
        i18n.t("agentChat.thread.roleClusterActivity.participants")
      )
    ).toBeInTheDocument()
    expect(
      document.querySelector("[data-agent-role-participant='data']")
    ).not.toBeNull()
    expect(
      document.querySelector("[data-agent-role-participant='ops']")
    ).not.toBeNull()
    expect(
      document.querySelector("[data-agent-role-participant='search']")
    ).not.toBeNull()
    expect(
      document.querySelector("[data-agent-portrait-asset='data-portrait']")
    ).not.toBeNull()
    expect(
      document.querySelector("[data-agent-portrait-asset='data-portrait']")
    ).toHaveClass("object-contain")
    expect(
      document
        .querySelector("[data-agent-portrait-asset='data-portrait']")
        ?.closest("[data-agent-portrait-frame='rect']")
    ).not.toBeNull()
    expect(
      document.querySelector("[data-agent-portrait-asset='ops-portrait']")
    ).not.toBeNull()
    expect(
      document.querySelector("[data-agent-portrait-asset='search-portrait']")
    ).not.toBeNull()
    expect(
      screen.getByText(
        new RegExp(
          i18n.t("agentChat.thread.roleClusterActivity.contribution.data")
        )
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        new RegExp(
          i18n.t("agentChat.thread.roleClusterActivity.contribution.ops")
        )
      )
    ).toBeInTheDocument()

    await user.click(
      screen.getByTitle(i18n.t("agentChat.thread.expandProcess"))
    )

    expect(
      screen.getAllByText(i18n.t("agentChat.thread.agents.data.name")).length
    ).toBeGreaterThan(0)
    expect(
      screen.getByText(i18n.t("agentChat.thread.roleClusterActivity.title"))
    ).toBeInTheDocument()
    const roleCluster = document.querySelector("[data-agent-role-cluster]")
    const progressBlock = document.querySelector(
      "[data-agent-process-progress]"
    )
    expect(roleCluster).not.toBeNull()
    expect(progressBlock).not.toBeNull()
    expect(
      roleCluster!.compareDocumentPosition(progressBlock!) &
        Node.DOCUMENT_POSITION_FOLLOWING
    ).toBeTruthy()
    expect(screen.getByText("Step 1")).toBeInTheDocument()
    expect(
      screen.getAllByText(i18n.t("agentChat.thread.agents.search.name")).length
    ).toBeGreaterThan(0)
    expect(
      screen.getByText(i18n.t("agentChat.thread.dispatchFacts.roleAudit.label"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.dispatchFacts.searchAudit.label")
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.dispatchFacts.materialAudit.label")
      )
    ).toBeInTheDocument()
  })

  it("expands the role work panel by default while streaming", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <InlineSwarmTaskCard
          isStreaming
          messageId="message-streaming"
          onCloseProcessPanel={vi.fn()}
          onOpenProcessPanel={vi.fn()}
          processPanelOpen={false}
          runtimeEvents={createStreamingGraphEvents()}
          showProcessPanelAction={false}
          task={task}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(i18n.t("agentChat.thread.roleClusterActivity.title"))
    ).toBeInTheDocument()
    expect(
      document.querySelector("[data-agent-role-card='data']")
    ).not.toBeNull()
    expect(
      document.querySelector("[data-agent-portrait-asset='data-portrait']")
    ).toHaveClass("agent-role-portrait__sprite--working")
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
      screen.queryByText(
        i18n.t("agentChat.thread.processKinds.multi_agent.badge")
      )
    ).not.toBeInTheDocument()
  })
})
