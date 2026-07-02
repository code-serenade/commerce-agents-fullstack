import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import type { ReactNode } from "react"
import { MemoryRouter, Route, Routes } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import {
  AgentChatContext,
  type AgentChatContextValue,
} from "../../agent-chat/context/agent-chat-context-value"
import AgentCapabilitiesPage from "../index"

const chatState = vi.hoisted(() => ({
  startNewConversation: vi.fn(),
}))

describe("AgentCapabilitiesPage", () => {
  it("starts a fresh chat from the capability page when chat context exists", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()

    render(
      <TestShell i18n={i18n} withChatContext>
        <Routes>
          <Route path="/agent-capabilities" element={<AgentCapabilitiesPage />} />
          <Route path="/commerce" element={<div>Q&A page</div>} />
        </Routes>
      </TestShell>
    )

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentCapabilities.header.action"),
      })
    )

    expect(chatState.startNewConversation).toHaveBeenCalledTimes(1)
    expect(screen.getByText("Q&A page")).toBeInTheDocument()
  })

  it("still opens and navigates back to Q&A when chat context is missing", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()

    render(
      <TestShell i18n={i18n}>
        <Routes>
          <Route path="/agent-capabilities" element={<AgentCapabilitiesPage />} />
          <Route path="/commerce" element={<div>Q&A page</div>} />
        </Routes>
      </TestShell>
    )

    expect(
      screen.getByRole("heading", {
        name: i18n.t("agentCapabilities.header.title"),
      })
    ).toBeInTheDocument()

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentCapabilities.header.action"),
      })
    )

    expect(screen.getByText("Q&A page")).toBeInTheDocument()
  })
})

function TestShell({
  children,
  i18n,
  withChatContext = false,
}: {
  children: ReactNode
  i18n: Parameters<typeof AgentChatTestI18nProvider>[0]["i18n"]
  withChatContext?: boolean
}) {
  const content = (
    <AgentChatTestI18nProvider i18n={i18n}>
      {children}
    </AgentChatTestI18nProvider>
  )

  return (
    <MemoryRouter initialEntries={["/agent-capabilities"]}>
      {withChatContext ? (
        <AgentChatContext.Provider
          value={{
            startNewConversation: chatState.startNewConversation,
          } as unknown as AgentChatContextValue}
        >
          {content}
        </AgentChatContext.Provider>
      ) : (
        content
      )}
    </MemoryRouter>
  )
}
