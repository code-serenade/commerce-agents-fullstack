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

  it("softens internal answer headings from legacy assistant output", async () => {
    const i18n = await createAgentChatTestI18n()
    const legacyFinalHeading = String.fromCodePoint(
      0x591a,
      0x667a,
      0x80fd,
      0x4f53,
      0x534f,
      0x4f5c,
      0x7ed3,
      0x8bba
    )
    const legacyReferenceHeading = String.fromCodePoint(
      0x5c97,
      0x4f4d,
      0x534f,
      0x4f5c
    )
    const legacyActionHeading = String.fromCodePoint(
      0x4e3b,
      0x5c97,
      0x6c47,
      0x603b,
      0x65b9,
      0x6848
    )

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <AgentChatThread
          activeProcessMessageId={null}
          isStreaming={false}
          messages={[
            {
              content:
                `## ${legacyFinalHeading}\nLegacy final content.\n\n## ${legacyReferenceHeading}\nLegacy reference content.\n\n## ${legacyActionHeading}\nLegacy action content.`,
              id: "assistant-legacy-headings",
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
      screen.getByText(i18n.t("agentChat.answer.safeHeadings.final"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.answer.safeHeadings.references"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.answer.safeHeadings.action"))
    ).toBeInTheDocument()
    expect(screen.queryByText(legacyFinalHeading)).not.toBeInTheDocument()
    expect(screen.queryByText(legacyReferenceHeading)).not.toBeInTheDocument()
    expect(screen.queryByText(legacyActionHeading)).not.toBeInTheDocument()
  })
})
