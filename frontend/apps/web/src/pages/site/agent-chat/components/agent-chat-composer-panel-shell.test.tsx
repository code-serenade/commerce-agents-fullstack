import { act, render, screen } from "@testing-library/react"
import type { ReactNode } from "react"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { AgentChatComposerPanelShell } from "./agent-chat-composer-panel-shell"
import {
  ComposerPanelFrame,
  ComposerPanelHeader,
} from "./composer-panel-layout"

describe("AgentChatComposerPanelShell", () => {
  it("keeps panel content scrollable with bottom breathing room", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerPanelShell maxHeight={240} open>
          <ComposerPanelFrame
            title={i18n.t("agentChat.composer.sourceContext.panelTitle")}
            description={i18n.t("agentChat.composer.sourceContext.panelHint")}
          >
            <div>Panel content</div>
          </ComposerPanelFrame>
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
    expect(shell).toHaveStyle({
      "--agent-chat-composer-panel-max-height": "240px",
    })
    expect(shell).not.toBe(scroll)
    expect(shell).toHaveClass("overflow-hidden")
    expect(content).toHaveClass("flex")
    expect(content).toHaveClass("flex-col")
    expect(content).toHaveClass("overflow-hidden")
    expect(content).toHaveClass("min-h-0")
    expect(scroll).toHaveClass("overflow-y-auto")
    expect(scroll).toHaveClass("overflow-x-hidden")
    expect(content).toHaveClass("p-4")
    expect(shell).toHaveClass("inset-x-0")
  })

  it("can render below the composer when the caller selects bottom placement", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerPanelShell
          maxHeight={240}
          open
          placement="bottom"
        >
          <div>Bottom panel content</div>
        </AgentChatComposerPanelShell>
      </TestProviders>
    )

    const shell = screen
      .getByText("Bottom panel content")
      .closest("[data-agent-chat-composer-panel-shell='true']")

    expect(shell).toHaveAttribute(
      "data-agent-chat-composer-panel-placement",
      "bottom"
    )
    expect(shell).toHaveClass("top-[calc(100%+1.25rem)]")
    expect(shell).toHaveClass("origin-top")
    expect(shell).not.toHaveClass("bottom-[calc(100%+1.25rem)]")
  })

  it("allows wide panels to use the available content viewport", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerPanelShell maxHeight={240} open size="wide">
          <div>Wide panel content</div>
        </AgentChatComposerPanelShell>
      </TestProviders>
    )

    const shell = screen
      .getByText("Wide panel content")
      .closest("[data-agent-chat-composer-panel-shell='true']")

    expect(shell).toHaveAttribute("data-agent-chat-composer-panel-size", "wide")
    expect(shell).toHaveClass("inset-x-0")
    expect(shell).toHaveClass(
      "sm:w-[min(calc(100dvw_-_var(--sidebar-width)_-_2rem),72rem)]"
    )
    expect(shell).toHaveClass("sm:max-w-[calc(100dvw_-_2rem)]")
  })

  it("renders close action inside the panel header area", async () => {
    const i18n = await createAgentChatTestI18n()
    const onClose = vi.fn()
    const panelTitle = i18n.t("agentChat.composer.sourceContext.panelTitle")
    const panelDescription = i18n.t(
      "agentChat.composer.sourceContext.panelHint"
    )

    render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerPanelShell maxHeight={240} onClose={onClose} open>
          <ComposerPanelHeader
            title={panelTitle}
            description={panelDescription}
          />
        </AgentChatComposerPanelShell>
      </TestProviders>
    )

    const closeButton = screen.getByRole("button", {
      name: i18n.t("actions.close", { ns: "common" }),
    })
    const content = screen
      .getByText(panelTitle)
      .closest("[data-agent-chat-composer-panel-content='true']")

    expect(content).toContainElement(closeButton)
  })

  it("keeps content mounted briefly while the panel collapses", async () => {
    vi.useFakeTimers()
    const i18n = await createAgentChatTestI18n()

    const { rerender } = render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerPanelShell maxHeight={240} open>
          <div>Panel content</div>
        </AgentChatComposerPanelShell>
      </TestProviders>
    )

    rerender(
      <TestProviders i18n={i18n}>
        <AgentChatComposerPanelShell maxHeight={240} open={false}>
          <div>Panel content</div>
        </AgentChatComposerPanelShell>
      </TestProviders>
    )

    const shell = screen
      .getByText("Panel content")
      .closest("[data-agent-chat-composer-panel-shell='true']")

    expect(shell).toHaveAttribute("aria-hidden", "true")
    expect(shell).toHaveClass("opacity-0")

    act(() => {
      vi.runAllTimers()
    })
    expect(screen.queryByText("Panel content")).not.toBeInTheDocument()
    vi.useRealTimers()
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
