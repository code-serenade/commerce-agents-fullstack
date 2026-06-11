import { render, screen, within } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import type { ReactNode } from "react"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { AgentChatComposerPanels } from "./agent-chat-composer-panels"
import {
  basePanelProps,
  runtimeSelection,
} from "./agent-chat-composer-test-fixtures"

describe("AgentChatComposer settings panel", () => {
  it("switches answer strategy, routing mode, collaboration mode, and roles", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()
    const onStrategyChange = vi.fn()
    const onRuntimeRoutingModeChange = vi.fn()
    const onRuntimeCollaborationModeChange = vi.fn()
    const onRuntimePrimaryRoleChange = vi.fn()
    const onToggleRuntimeRole = vi.fn()
    const { rerender } = render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerPanels
          {...basePanelProps()}
          onRuntimeCollaborationModeChange={onRuntimeCollaborationModeChange}
          onRuntimePrimaryRoleChange={onRuntimePrimaryRoleChange}
          onRuntimeRoutingModeChange={onRuntimeRoutingModeChange}
          onStrategyChange={onStrategyChange}
          onToggleRuntimeRole={onToggleRuntimeRole}
          settingsOpen
        />
      </TestProviders>
    )

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.strategy.options.quick.label"),
      })
    )
    expect(onStrategyChange).toHaveBeenCalledWith("quick")
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.composer.runtime.routingModes.fast.label"),
      })
    )
    expect(onRuntimeRoutingModeChange).toHaveBeenCalledWith("fast")
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.composer.runtime.roleModes.manual.label"),
      })
    )
    expect(onRuntimeCollaborationModeChange).toHaveBeenCalledWith("manual")

    rerender(
      <TestProviders i18n={i18n}>
        <AgentChatComposerPanels
          {...basePanelProps()}
          onRuntimeCollaborationModeChange={onRuntimeCollaborationModeChange}
          onRuntimePrimaryRoleChange={onRuntimePrimaryRoleChange}
          onRuntimeRoutingModeChange={onRuntimeRoutingModeChange}
          onStrategyChange={onStrategyChange}
          onToggleRuntimeRole={onToggleRuntimeRole}
          runtimeSelection={{
            ...runtimeSelection,
            collaborationMode: "manual",
            selectedRoles: ["ops", "data"],
          }}
          settingsOpen
        />
      </TestProviders>
    )

    const dataRoleRow = screen.getByText(
      i18n.t("agentChat.composer.runtime.roleLabels.data.name")
    ).closest("div")
    expect(dataRoleRow).not.toBeNull()
    await user.click(
      within(dataRoleRow as HTMLElement).getByRole("button", {
        name: i18n.t("agentChat.composer.runtime.primaryShort"),
      })
    )
    expect(onRuntimePrimaryRoleChange).toHaveBeenCalledWith("data")

    const creativeRoleRow = screen.getByText(
      i18n.t("agentChat.composer.runtime.roleLabels.creative.name")
    ).closest("div")
    expect(creativeRoleRow).not.toBeNull()
    await user.click(
      within(creativeRoleRow as HTMLElement).getByRole("button", {
        name: i18n.t("agentChat.composer.runtime.supportShort"),
      })
    )
    expect(onToggleRuntimeRole).toHaveBeenCalledWith("creative")
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
