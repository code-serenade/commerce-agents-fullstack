import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { getAgentChatStrategy } from "../domain/agent-chat-strategy"
import { AgentChatThread } from "./agent-chat-thread"

describe("AgentChatThread", () => {
  it("does not show the strategy pill before the user starts chatting", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <AgentChatThread
          activeProcessMessageId={null}
          isStreaming={false}
          messages={[
            {
              content: i18n.t("agentChat.welcomeMessage"),
              id: "welcome",
              role: "assistant",
            },
          ]}
          onCloseProcessPanel={vi.fn()}
          onOpenProcessPanel={vi.fn()}
          runtimeEvents={[]}
          selectedStrategy={getAgentChatStrategy("learning")}
          showProcessPanelAction
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.queryByText(i18n.t("agentChat.strategy.current"))
    ).not.toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.welcomeMessage"))
    ).toBeInTheDocument()
  })
})
