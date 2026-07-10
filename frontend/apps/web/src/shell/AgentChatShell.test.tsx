import { configureStore } from "@reduxjs/toolkit"
import { render, screen, within } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import type { ReactNode } from "react"
import { Provider } from "react-redux"
import { MemoryRouter, useLocation } from "react-router"
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

    const historyLabel = screen.getAllByText(
      i18n.t("agentChat.shell.sections.recents")
    )[0]

    const newChatButton = screen
      .getAllByRole("button", {
        name: i18n.t("agentChat.sidebar.newChat"),
      })
      .find((button) =>
        Boolean(
          button.compareDocumentPosition(historyLabel) &
          Node.DOCUMENT_POSITION_FOLLOWING
        )
      )
    expect(newChatButton).toBeDefined()
    if (!newChatButton) {
      throw new Error("Expected new chat button above history")
    }
    expect(
      newChatButton.compareDocumentPosition(historyLabel) &
        Node.DOCUMENT_POSITION_FOLLOWING
    ).toBeTruthy()
    const firstHistoryButton = screen.getByRole("button", {
      name: chatState.conversationTitles[0],
    })
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.navigation.questionIdeas"),
      })
    ).toBeInTheDocument()
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
      screen.getAllByText(i18n.t("agentChat.shell.sections.recents")).length
    ).toBeGreaterThan(0)
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

  it("hides history and training navigation tabs for guests", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n} loggedIn={false}>
        <AgentChatShell>
          <main>Guest content</main>
        </AgentChatShell>
      </TestProviders>
    )

    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.sidebar.newChat"),
      })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.shell.navigation.questionIdeas"),
      })
    ).toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.navigation.productBoard"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.navigation.materials"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.navigation.campaigns"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.tabs.chat"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryAllByText(i18n.t("agentChat.shell.sections.recents"))
    ).toHaveLength(0)
    expect(screen.getByText("Guest content")).toBeInTheDocument()
  }, 15000)

  it("navigates guests to login directly from the footer account entry", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()
    const locationSpy = vi.fn()

    render(
      <TestProviders
        i18n={i18n}
        initialEntries={["/commerce"]}
        loggedIn={false}
        onLocationChange={locationSpy}
      >
        <AgentChatShell>
          <main>Guest footer content</main>
        </AgentChatShell>
      </TestProviders>
    )

    const loginEntry = screen.getByText(i18n.t("web.shell.account.loginCta"))
    await user.click(loginEntry.closest("button") as HTMLButtonElement)

    expect(locationSpy).toHaveBeenLastCalledWith("/login")
    expect(
      screen.queryByRole("menuitem", {
        name: i18n.t("web.shell.account.login"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("menuitem", {
        name: i18n.t("web.shell.account.register"),
      })
    ).not.toBeInTheDocument()
  }, 15000)

  it("folds common entries after the configured visible count", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()

    render(
      <TestProviders i18n={i18n}>
        <AgentChatShell>
          <main>Overflow content</main>
        </AgentChatShell>
      </TestProviders>
    )

    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.shell.navigation.productBoard"),
      })
    ).toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.navigation.materials"),
      })
    ).not.toBeInTheDocument()

    await user.click(
      screen.getByRole("button", { name: i18n.t("agentChat.sidebar.more") })
    )

    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.sidebar.less"),
      })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.shell.navigation.materials"),
      })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.shell.navigation.campaigns"),
      })
    ).toBeInTheDocument()
  }, 15000)

  it("switches logged-in users between chat and training navigation", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()

    render(
      <TestProviders i18n={i18n}>
        <AgentChatShell>
          <main>Training tab content</main>
        </AgentChatShell>
      </TestProviders>
    )

    expect(
      screen.getAllByText(i18n.t("agentChat.shell.sections.recents")).length
    ).toBeGreaterThan(0)

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.shell.tabs.training"),
      })
    )

    expect(
      screen.getByRole("button", {
        name: i18n.t("web.shell.navigation.approvals"),
      })
    ).toBeInTheDocument()
  }, 15000)

  it("opens the training submenu instead of common entries on mobile", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()

    render(
      <TestProviders i18n={i18n}>
        <AgentChatShell>
          <main>Mobile navigation content</main>
        </AgentChatShell>
      </TestProviders>
    )

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("web.shell.navigation.workspace"),
      })
    )

    expect(
      screen.getByRole("link", {
        name: i18n.t("web.shell.navigation.products"),
      })
    ).toBeInTheDocument()
    expect(
      screen.queryByRole("link", {
        name: i18n.t("agentChat.sidebar.newChat"),
      })
    ).not.toBeInTheDocument()
  }, 15000)

  it("only highlights mobile history while a conversation route is active", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n} initialEntries={["/products"]}>
        <AgentChatShell>
          <main>Products content</main>
        </AgentChatShell>
      </TestProviders>
    )

    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.shell.sections.recents"),
      })
    ).not.toHaveAttribute("aria-current", "page")
  }, 15000)
})

function TestProviders({
  children,
  i18n,
  initialEntries = ["/commerce"],
  loggedIn = true,
  onLocationChange,
}: {
  children: ReactNode
  i18n: Parameters<typeof AgentChatTestI18nProvider>[0]["i18n"]
  initialEntries?: string[]
  loggedIn?: boolean
  onLocationChange?: (pathname: string) => void
}) {
  const store = configureStore({
    reducer: {
      auth: authReducer,
      webAccess: accessReducer,
      requestError: requestErrorReducer,
    },
    preloadedState: {
      auth: loggedIn
        ? {
            isLogin: true,
            token: "test-token",
            user: {
              avatar: "",
              display_id: "test-user",
              email: "test@example.com",
              id: "user-1",
              name: "Test User",
            },
          }
        : {
            isLogin: false,
            token: null,
            user: null,
          },
    },
  })

  return (
    <MemoryRouter initialEntries={initialEntries}>
      <Provider store={store}>
        <AgentChatTestI18nProvider i18n={i18n}>
          {onLocationChange ? (
            <LocationProbe onLocationChange={onLocationChange} />
          ) : null}
          {children}
        </AgentChatTestI18nProvider>
      </Provider>
    </MemoryRouter>
  )
}

function LocationProbe({
  onLocationChange,
}: {
  onLocationChange: (pathname: string) => void
}) {
  const location = useLocation()
  onLocationChange(location.pathname)
  return null
}
