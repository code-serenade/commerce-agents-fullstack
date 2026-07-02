import { render, screen } from "@testing-library/react"
import type { ReactNode } from "react"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { getAgentChatStrategy } from "../domain/agent-chat-strategy"
import { AgentChatThread } from "./agent-chat-thread"

describe("AgentChatThread avatar", () => {
  it("uses the commerce copilot pixel avatar for assistant replies", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatThreadAvatarTestShell i18n={i18n}>
        <AgentChatThread
          activeProcessMessageId={null}
          isStreaming={false}
          messages={[
            {
              content: "A practical answer",
              id: "assistant-1",
              role: "assistant",
            },
          ]}
          onCloseProcessPanel={vi.fn()}
          onOpenProcessPanel={vi.fn()}
          runtimeEvents={[]}
          selectedStrategy={getAgentChatStrategy("learning")}
          showProcessPanelAction
        />
      </AgentChatThreadAvatarTestShell>
    )

    expect(
      screen.getByLabelText(
        i18n.t("agentChat.thread.assistantAvatar.ariaLabelReady")
      )
    ).toBeInTheDocument()
    expect(
      document.querySelector("[data-agent-portrait-asset='box-portrait']")
    ).toHaveAttribute("data-agent-portrait-source", "static")
  })

  it("shows the commerce copilot avatar as working while streaming", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatThreadAvatarTestShell i18n={i18n}>
        <AgentChatThread
          activeProcessMessageId={null}
          isStreaming
          messages={[
            {
              content: "",
              id: "assistant-streaming",
              role: "assistant",
              status: "streaming",
            },
          ]}
          onCloseProcessPanel={vi.fn()}
          onOpenProcessPanel={vi.fn()}
          runtimeEvents={[]}
          selectedStrategy={getAgentChatStrategy("learning")}
          showProcessPanelAction
        />
      </AgentChatThreadAvatarTestShell>
    )

    expect(
      screen.getByLabelText(
        i18n.t("agentChat.thread.assistantAvatar.ariaLabelWorking")
      )
    ).toBeInTheDocument()
    expect(
      document.querySelector("[data-agent-portrait-asset='box-portrait']")
    ).toHaveClass("agent-role-portrait__sprite--working")
  })
})

function AgentChatThreadAvatarTestShell({
  children,
  i18n,
}: {
  children: ReactNode
  i18n: Parameters<typeof AgentChatTestI18nProvider>[0]["i18n"]
}) {
  return (
    <MemoryRouter>
      <AgentChatTestI18nProvider i18n={i18n}>
        {children}
      </AgentChatTestI18nProvider>
    </MemoryRouter>
  )
}
