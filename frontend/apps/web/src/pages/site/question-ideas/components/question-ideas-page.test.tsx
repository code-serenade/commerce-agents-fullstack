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
import QuestionIdeasPage from "../index"

const chatState = vi.hoisted(() => ({
  setInput: vi.fn(),
}))

describe("QuestionIdeasPage", () => {
  it("fills a selected question idea into the chat input and returns to Q&A", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()
    const prompt = i18n.t("agentChat.thread.studentTaskEntries.start.prompt")

    render(
      <TestShell i18n={i18n}>
        <Routes>
          <Route path="/question-ideas" element={<QuestionIdeasPage />} />
          <Route path="/commerce" element={<div>Q&A page</div>} />
        </Routes>
      </TestShell>
    )

    expect(
      screen.getByRole("heading", { name: i18n.t("questionIdeas.header.title") })
    ).toBeInTheDocument()

    await user.click(screen.getByText(prompt))

    expect(chatState.setInput).toHaveBeenCalledWith(prompt)
    expect(screen.getByText("Q&A page")).toBeInTheDocument()
  })

  it("still opens when the chat provider is missing", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()
    const prompt = i18n.t("agentChat.thread.studentTaskEntries.start.prompt")

    render(
      <MemoryRouter initialEntries={["/question-ideas"]}>
        <AgentChatTestI18nProvider i18n={i18n}>
          <Routes>
            <Route path="/question-ideas" element={<QuestionIdeasPage />} />
            <Route path="/commerce" element={<div>Q&A page</div>} />
          </Routes>
        </AgentChatTestI18nProvider>
      </MemoryRouter>
    )

    expect(
      screen.getByRole("heading", { name: i18n.t("questionIdeas.header.title") })
    ).toBeInTheDocument()

    await user.click(screen.getByText(prompt))

    expect(screen.getByText("Q&A page")).toBeInTheDocument()
  })
})

function TestShell({
  children,
  i18n,
}: {
  children: ReactNode
  i18n: Parameters<typeof AgentChatTestI18nProvider>[0]["i18n"]
}) {
  return (
    <MemoryRouter initialEntries={["/question-ideas"]}>
      <AgentChatContext.Provider
        value={{ setInput: chatState.setInput } as unknown as AgentChatContextValue}
      >
        <AgentChatTestI18nProvider i18n={i18n}>
          {children}
        </AgentChatTestI18nProvider>
      </AgentChatContext.Provider>
    </MemoryRouter>
  )
}
