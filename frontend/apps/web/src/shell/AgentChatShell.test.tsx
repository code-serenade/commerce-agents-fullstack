import { configureStore } from "@reduxjs/toolkit"
import { render, screen, within } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import type { ReactNode } from "react"
import { Provider } from "react-redux"
import { MemoryRouter } from "react-router"
import { beforeEach, describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import accessReducer from "@/store/accessSlice"
import authReducer from "@/store/authSlice"
import requestErrorReducer from "@/store/requestErrorSlice"
import type { AgentChatContextValue } from "@/pages/site/agent-chat/context/agent-chat-context-value"
import { AgentChatShell } from "./AgentChatShell"

const chatState = vi.hoisted(() => ({
  conversationTitles: [
    ["First", "customer", "thread"].join(" "),
    ["Second", "customer", "thread"].join(" "),
  ],
  startNewConversation: vi.fn(),
  selectConversation: vi.fn(),
  renameConversation: vi.fn(),
  deleteConversation: vi.fn(),
}))

vi.mock("@/pages/site/agent-chat/use-agent-chat-page", () => ({
  useAgentChatPage: () =>
    ({
      conversationId: "conversation-1",
      conversations: [
        {
          id: "conversation-1",
          title: chatState.conversationTitles[0],
          updatedAt: "2026-06-11T10:00:00Z",
        },
        {
          id: "conversation-2",
          title: chatState.conversationTitles[1],
          updatedAt: "2026-06-11T11:00:00Z",
        },
      ],
      deleteConversation: chatState.deleteConversation,
      isStreaming: false,
      renameConversation: chatState.renameConversation,
      selectConversation: chatState.selectConversation,
      startNewConversation: chatState.startNewConversation,
    }) as unknown as AgentChatContextValue,
}))

describe("AgentChatShell", () => {
  beforeEach(() => {
    chatState.startNewConversation.mockClear()
    chatState.selectConversation.mockClear()
    Object.defineProperty(window, "matchMedia", {
      configurable: true,
      value: vi.fn().mockImplementation((query: string) => ({
        addEventListener: vi.fn(),
        addListener: vi.fn(),
        dispatchEvent: vi.fn(),
        matches: false,
        media: query,
        onchange: null,
        removeEventListener: vi.fn(),
        removeListener: vi.fn(),
      })),
    })
  })

  it("shows function entries above history and keeps history entries selectable", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()

    render(
      <TestProviders i18n={i18n}>
        <AgentChatShell>
          <main>Shell content</main>
        </AgentChatShell>
      </TestProviders>
    )

    const historyToggle = screen.getByRole("button", {
      name: i18n.t("agentChat.sidebar.collapseHistory"),
    })

    const newChatButton = screen
      .getAllByRole("button", {
        name: i18n.t("agentChat.sidebar.newChat"),
      })
      .find((button) =>
        Boolean(
          button.compareDocumentPosition(historyToggle) &
          Node.DOCUMENT_POSITION_FOLLOWING
        )
      )
    expect(newChatButton).toBeDefined()
    if (!newChatButton) {
      throw new Error("Expected new chat button above history")
    }
    expect(
      newChatButton.compareDocumentPosition(historyToggle) &
        Node.DOCUMENT_POSITION_FOLLOWING
    ).toBeTruthy()
    const firstHistoryButton = screen.getByRole("button", {
      name: chatState.conversationTitles[0],
    })
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.navigation.questionIdeas"),
      })
    ).not.toBeInTheDocument()
    expect(newChatButton.getAttribute("data-size")).toBe(
      firstHistoryButton.getAttribute("data-size")
    )

    const newChatText = within(newChatButton).getByText(
      i18n.t("agentChat.sidebar.newChat")
    )
    await user.click(newChatText.closest("button") as HTMLButtonElement)
    expect(chatState.startNewConversation).toHaveBeenCalledTimes(1)

    await user.click(
      screen.getByRole("button", { name: chatState.conversationTitles[1] })
    )
    expect(chatState.selectConversation).toHaveBeenCalledWith("conversation-2")
  }, 15000)

  it("keeps history visible on function pages so conversations are not hidden", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n} initialEntries={["/question-ideas"]}>
        <AgentChatShell>
          <main>Question ideas content</main>
        </AgentChatShell>
      </TestProviders>
    )

    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.sidebar.collapseHistory"),
      })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.sidebar.newChat"),
      })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", { name: chatState.conversationTitles[0] })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", { name: chatState.conversationTitles[1] })
    ).toBeInTheDocument()
    expect(screen.getByText("Question ideas content")).toBeInTheDocument()
  }, 15000)
})

function TestProviders({
  children,
  i18n,
  initialEntries = ["/commerce"],
}: {
  children: ReactNode
  i18n: Parameters<typeof AgentChatTestI18nProvider>[0]["i18n"]
  initialEntries?: string[]
}) {
  const store = configureStore({
    reducer: {
      auth: authReducer,
      webAccess: accessReducer,
      requestError: requestErrorReducer,
    },
  })

  return (
    <MemoryRouter initialEntries={initialEntries}>
      <Provider store={store}>
        <AgentChatTestI18nProvider i18n={i18n}>
          {children}
        </AgentChatTestI18nProvider>
      </Provider>
    </MemoryRouter>
  )
}
