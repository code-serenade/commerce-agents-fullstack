import { configureStore } from "@reduxjs/toolkit"
import { fireEvent, render, screen, within } from "@testing-library/react"
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
  conversations: [
    {
      id: "conversation-1",
      title: ["First", "customer", "thread"].join(" "),
      updatedAt: "2026-06-11T10:00:00Z",
    },
    {
      id: "conversation-2",
      title: ["Second", "customer", "thread"].join(" "),
      updatedAt: "2026-06-11T11:00:00Z",
    },
  ],
  startNewConversation: vi.fn(),
  selectConversation: vi.fn(),
  renameConversation: vi.fn(),
  deleteConversation: vi.fn(),
}))

const authDialogState = vi.hoisted(() => ({
  requestLogin: vi.fn(),
}))

vi.mock("@/pages/site/agent-chat/use-agent-chat-page", () => ({
  useAgentChatPage: () =>
    ({
      conversationId: "conversation-1",
      conversations: chatState.conversations,
      deleteConversation: chatState.deleteConversation,
      isStreaming: false,
      renameConversation: chatState.renameConversation,
      selectConversation: chatState.selectConversation,
      startNewConversation: chatState.startNewConversation,
    }) as unknown as AgentChatContextValue,
}))

vi.mock("@/components/auth/AuthDialogProvider", () => ({
  useAuthDialog: () => ({
    requestLogin: authDialogState.requestLogin,
  }),
}))

describe("AgentChatShell", () => {
  beforeEach(() => {
    chatState.conversations = [
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
    ]
    chatState.startNewConversation.mockClear()
    chatState.selectConversation.mockClear()
    authDialogState.requestLogin.mockClear()
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
    const locationSpy = vi.fn()

    render(
      <TestProviders i18n={i18n} onLocationChange={locationSpy}>
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
    const historyToggle = screen
      .getAllByRole("button", {
        name: i18n.t("agentChat.shell.sections.recents"),
      })
      .find((button) => button.hasAttribute("aria-expanded"))
    expect(historyToggle).toBeDefined()
    expect(historyToggle).toHaveAttribute("aria-expanded", "true")

    const firstHistoryButton = screen.getByRole("button", {
      name: chatState.conversationTitles[0],
    })
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.productBoard"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.materials"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.campaigns"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.sidebar.more"),
      })
    ).toBeInTheDocument()
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
    expect(locationSpy).toHaveBeenLastCalledWith("/commerce/conversation-2")

    await user.click(historyToggle as HTMLButtonElement)
    expect(historyToggle).toHaveAttribute("aria-expanded", "false")
    expect(
      screen.queryByRole("button", { name: chatState.conversationTitles[0] })
    ).not.toBeInTheDocument()
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
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.sidebar.newChat"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.getByRole("button", { name: chatState.conversationTitles[0] })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", { name: chatState.conversationTitles[1] })
    ).toBeInTheDocument()
    expect(screen.getByText("Question ideas content")).toBeInTheDocument()
  }, 15000)

  it("shows the same navigation entries for guests", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()

    render(
      <TestProviders i18n={i18n} loggedIn={false}>
        <AgentChatShell>
          <main>Guest content</main>
        </AgentChatShell>
      </TestProviders>
    )

    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.sidebar.newChat"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen
        .queryAllByText(i18n.t("agentChat.shell.sections.workspace"))
        .filter((element) => !element.closest("button,a"))
    ).toHaveLength(0)
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.productBoard"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.materials"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.campaigns"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.tabs.chat"),
      })
    ).not.toBeInTheDocument()
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.sidebar.more"),
      })
    )
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.questionIdeas"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.getAllByText(i18n.t("agentChat.shell.sections.recents")).length
    ).toBeGreaterThan(0)
    expect(screen.getByText("Guest content")).toBeInTheDocument()
  }, 15000)

  it("opens the login dialog instead of navigating to login for protected guest menu entries", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()
    const locationSpy = vi.fn()

    render(
      <TestProviders
        i18n={i18n}
        loggedIn={false}
        onLocationChange={locationSpy}
      >
        <AgentChatShell>
          <main>Protected guest menu content</main>
        </AgentChatShell>
      </TestProviders>
    )

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.sidebar.more"),
      })
    )
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("web.shell.navigation.agentOutput"),
      })
    )

    expect(authDialogState.requestLogin).toHaveBeenCalledWith({
      pathname: "/agent-output",
    })
    expect(locationSpy).toHaveBeenLastCalledWith("/commerce")
  }, 15000)

  it("shows a plain empty history message when signed in with no conversations", async () => {
    const i18n = await createAgentChatTestI18n()
    chatState.conversations = []

    render(
      <TestProviders i18n={i18n}>
        <AgentChatShell>
          <main>Empty history content</main>
        </AgentChatShell>
      </TestProviders>
    )

    const emptyHistory = screen.getByText(i18n.t("agentChat.sidebar.empty"))
    expect(emptyHistory).toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.sidebar.empty"),
      })
    ).not.toBeInTheDocument()
  }, 15000)

  it("asks guests to sign in with a plain history sync message", async () => {
    const i18n = await createAgentChatTestI18n()
    chatState.conversations = []

    render(
      <TestProviders i18n={i18n} loggedIn={false}>
        <AgentChatShell>
          <main>Guest empty history content</main>
        </AgentChatShell>
      </TestProviders>
    )

    const guestEmptyHistory = screen.getByText(
      i18n.t("agentChat.sidebar.guestEmpty")
    )
    expect(guestEmptyHistory).toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.sidebar.guestEmpty"),
      })
    ).not.toBeInTheDocument()
  }, 15000)

  it("requests login directly from the guest footer account entry", async () => {
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

    expect(authDialogState.requestLogin).toHaveBeenCalledTimes(1)
    expect(locationSpy).toHaveBeenLastCalledWith("/commerce")
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

  it("starts a new conversation with the Ctrl+K shortcut", async () => {
    const i18n = await createAgentChatTestI18n()
    const locationSpy = vi.fn()

    render(
      <TestProviders
        i18n={i18n}
        initialEntries={["/products"]}
        onLocationChange={locationSpy}
      >
        <AgentChatShell>
          <main>Shortcut content</main>
        </AgentChatShell>
      </TestProviders>
    )

    fireEvent.keyDown(window, { key: "k", ctrlKey: true })

    expect(chatState.startNewConversation).toHaveBeenCalledTimes(1)
    expect(locationSpy).toHaveBeenLastCalledWith("/commerce")
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.sidebar.newChat"),
      }).length
    ).toBeGreaterThan(0)
  }, 15000)

  it("keeps common entries compact and places the overflow control after them", async () => {
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
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.productBoard"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.sidebar.more"),
      })
    ).toBeInTheDocument()
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.materials"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.campaigns"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.navigation.capabilities"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.navigation.questionIdeas"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.agentOutput"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.workspaces"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.workspaces"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.platformConnect"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.approvals"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.operations"),
      })
    ).not.toBeInTheDocument()

    const moreButton = screen.getByRole("button", {
      name: i18n.t("agentChat.sidebar.more"),
    })
    await user.click(moreButton)

    const lessButton = screen.getByRole("button", {
      name: i18n.t("agentChat.sidebar.less"),
    })
    const capabilitiesButton = screen.getByRole("button", {
      name: i18n.t("agentChat.shell.navigation.capabilities"),
    })
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.shell.navigation.questionIdeas"),
      })
    ).toBeInTheDocument()
    const agentOutputButton = screen.getByRole("button", {
      name: i18n.t("web.shell.navigation.agentOutput"),
    })
    expect(
      lessButton.compareDocumentPosition(capabilitiesButton) &
        Node.DOCUMENT_POSITION_FOLLOWING
    ).toBeTruthy()
    expect(
      lessButton.compareDocumentPosition(agentOutputButton) &
        Node.DOCUMENT_POSITION_FOLLOWING
    ).toBeTruthy()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.home"),
      })
    ).not.toBeInTheDocument()

    await user.click(lessButton)

    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.productBoard"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.materials"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.campaigns"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.navigation.capabilities"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.navigation.questionIdeas"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.agentOutput"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.sidebar.more"),
      })
    ).toBeInTheDocument()
  }, 15000)

  it("keeps the SYF-style single sidebar while hiding removed training entries", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()

    render(
      <TestProviders i18n={i18n}>
        <AgentChatShell>
          <main>Merged navigation content</main>
        </AgentChatShell>
      </TestProviders>
    )

    expect(
      screen.getAllByText(i18n.t("agentChat.shell.sections.recents")).length
    ).toBeGreaterThan(0)
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.sidebar.newChat"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.tabs.chat"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.shell.tabs.training"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.shell.sections.recents"),
        expanded: true,
      })
    ).toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.approvals"),
      })
    ).not.toBeInTheDocument()

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.sidebar.more"),
      })
    )
    expect(
      screen.getByRole("button", {
        name: i18n.t("web.shell.navigation.agentOutput"),
      })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.shell.sections.recents"),
        expanded: true,
      })
    ).toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.approvals"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.workspaces"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.platformConnect"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.operations"),
      })
    ).not.toBeInTheDocument()
  }, 15000)

  it("keeps the merged navigation visible when refreshing a training route", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n} initialEntries={["/workspaces/workspace-1"]}>
        <AgentChatShell>
          <main>Workspace detail content</main>
        </AgentChatShell>
      </TestProviders>
    )

    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.sidebar.newChat"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.workspaces"),
      })
    ).not.toBeInTheDocument()

    expect(
      screen.queryByRole("button", {
        name: i18n.t("web.shell.navigation.workspaces"),
      })
    ).not.toBeInTheDocument()
  }, 15000)

  it("keeps shared training routes in the merged common navigation", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n} initialEntries={["/products"]}>
        <AgentChatShell>
          <main>Products content</main>
        </AgentChatShell>
      </TestProviders>
    )

    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.sidebar.newChat"),
      }).length
    ).toBeGreaterThan(0)
    expect(
      screen.getAllByRole("button", {
        name: i18n.t("agentChat.shell.navigation.productBoard"),
      }).length
    ).toBeGreaterThan(0)
  }, 15000)

  it("opens the merged common entries on mobile", async () => {
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
        name: i18n.t("agentChat.shell.sections.workspace"),
      })
    )

    expect(
      screen.getByRole("link", {
        name: i18n.t("agentChat.shell.navigation.productBoard"),
      })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("link", {
        name: i18n.t("agentChat.sidebar.newChat"),
      })
    ).toBeInTheDocument()
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

    const mobileHistoryButton = screen
      .getAllByRole("button", {
        name: i18n.t("agentChat.shell.sections.recents"),
      })
      .find((button) => !button.hasAttribute("aria-label"))
    expect(mobileHistoryButton).toBeDefined()
    expect(mobileHistoryButton).not.toHaveAttribute("aria-current", "page")
  }, 15000)

  it("keeps report pages on the fixed split-view viewport", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n} initialEntries={["/agent-output"]}>
        <AgentChatShell>
          <main>Report content</main>
        </AgentChatShell>
      </TestProviders>
    )

    const contentFrame = screen.getByText("Report content").parentElement
    expect(contentFrame).toHaveClass("size-full")
    expect(contentFrame).toHaveClass("min-h-0")
    expect(contentFrame).toHaveClass("flex-1")
  }, 15000)

  it("keeps commerce pages on the fixed inner-scroll viewport", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n} initialEntries={["/products"]}>
        <AgentChatShell>
          <main>Products content</main>
        </AgentChatShell>
      </TestProviders>
    )

    const contentFrame = screen.getByText("Products content").parentElement
    expect(contentFrame).toHaveClass("size-full")
    expect(contentFrame).toHaveClass("min-h-0")
    expect(contentFrame).toHaveClass("flex-1")
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
