import { render, screen } from "@testing-library/react"
import type { ReactNode } from "react"
import { MemoryRouter } from "react-router"
import { describe, expect, it } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { AgentChatComposerPanelShell } from "./agent-chat-composer-panel-shell"

describe("AgentChatComposerPanelShell", () => {
  it("keeps panel content scrollable with bottom breathing room", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerPanelShell maxHeight={240}>
          <div>Panel content</div>
        </AgentChatComposerPanelShell>
      </TestProviders>
    )

    const shell = screen
      .getByText("Panel content")
      .closest("[data-agent-chat-composer-panel-shell='true']")
    const scroll = screen
      .getByText("Panel content")
      .closest("[data-agent-chat-composer-panel-scroll='true']")
    const content = screen
      .getByText("Panel content")
      .closest("[data-agent-chat-composer-panel-content='true']")

    expect(shell).not.toBeNull()
    expect(scroll).not.toBeNull()
    expect(content).not.toBeNull()
    expect(shell).toHaveStyle({ maxHeight: "240px" })
    expect(scroll).toHaveStyle({ maxHeight: "240px" })
    expect(shell).toBe(scroll)
    expect(scroll).toHaveClass("overflow-y-auto")
    expect(scroll).toHaveClass("overflow-x-hidden")
    expect(content).toHaveClass("pb-16")
  })
})

function TestProviders({
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
