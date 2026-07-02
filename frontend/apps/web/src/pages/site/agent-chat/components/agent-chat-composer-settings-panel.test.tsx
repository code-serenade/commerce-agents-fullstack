import { render, screen, within } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import type { ReactNode } from "react"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { AgentChatComposerPanels } from "./agent-chat-composer-panels"
import { AgentChatComposerToolbar } from "./agent-chat-composer-toolbar"
import zhMessages from "@/../lang/zhCN/messages"
import {
  baseToolbarProps,
  basePanelProps,
  runtimeSelection,
} from "./agent-chat-composer-test-fixtures"

describe("AgentChatComposer settings panel", () => {
  const copy = {
    activity: textFromCodes(27963, 21160),
    agentBrand: textFromCodes(30005, 21830, 22810, 26234, 33021, 20307, 24179, 21488),
    answerStyle: textFromCodes(22238, 31572, 26041, 24335),
    filesAndImages: textFromCodes(36164, 26009, 21644, 22270, 29255),
    multiAgentMode: textFromCodes(26234, 33021, 20307, 35843, 29992, 26041, 24335),
    multiAgentJudgment: textFromCodes(22810, 23703, 20301, 21028, 26029),
    joinActivityReference: textFromCodes(21152, 20837, 27963, 21160, 21442, 32771),
    joinProductReference: textFromCodes(21152, 20837, 21830, 21697, 21442, 32771),
    manualMultipleAgents: textFromCodes(25351, 23450, 22810, 20010, 23703, 20301),
    manualSingleAgent: textFromCodes(25351, 23450, 19968, 20010, 23703, 20301),
    oldAgentBrand: textFromCodes(30005, 21830, 23567, 21442, 35851),
    oldAutoJudge: textFromCodes(25105, 26469, 21028, 26029),
    oldFocusOneReference: textFromCodes(37325, 28857, 30475, 19968, 22788),
    oldFocusPerspective: textFromCodes(37325, 28857, 35270, 35282),
    oldMultiReference: textFromCodes(22810, 34917, 20960, 22788),
    oldProductBoard: textFromCodes(30475, 21830, 21697),
    oldReferenceMode: textFromCodes(21442, 32771, 26041, 24335),
    oldSelectedReferences: textFromCodes(26412, 36718, 24590, 20040, 21442, 32771),
    product: textFromCodes(21830, 21697),
    productAnalysis: textFromCodes(36873, 21697, 20998, 26512),
    selectedAgentRoles: textFromCodes(36873, 25321, 21442, 19982, 23703, 20301),
    systemAutoAgents: textFromCodes(31995, 32479, 33258, 21160, 36873),
    singleAgentJudgment: textFromCodes(21333, 23703, 20301, 21028, 26029),
    thinkTogether: textFromCodes(19968, 36215, 24819),
    unselectedActivity: textFromCodes(26410, 36873, 27963, 21160),
    unselectedProduct: textFromCodes(26410, 36873, 21830, 21697),
  }

  it("keeps the compact toolbar free of default runtime summary clutter", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerToolbar {...baseToolbarProps()} />
      </TestProviders>
    )

    expect(
      screen.queryByText(i18n.t("agentChat.composer.settings.selectedPrefix"))
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.composer.settings.hideSelection"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.composer.settings.showSelection"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.getByRole("button", {
        name: new RegExp(i18n.t("agentChat.strategy.options.learning.label")),
      })
    ).toBeInTheDocument()
    expect(screen.queryByText(copy.thinkTogether)).not.toBeInTheDocument()
  })

  it("uses customer-facing toolbar labels for files, products, campaigns, and answer style", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerToolbar
          {...baseToolbarProps()}
          availableCampaigns={[]}
          availableProducts={[]}
          selectedCampaigns={[]}
          selectedProducts={[]}
          selectedSources={[]}
        />
      </TestProviders>
    )

    expect(
      screen.getByText(i18n.t("agentChat.composer.sourceContext.compactLabel"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.composer.productContext.compactLabel"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.composer.campaignContext.compactLabel"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.composer.settings.compactLabel"))
    ).toBeInTheDocument()
    expect(screen.queryByText(copy.unselectedProduct)).not.toBeInTheDocument()
    expect(screen.queryByText(copy.unselectedActivity)).not.toBeInTheDocument()
    expect(screen.queryByText(copy.thinkTogether)).not.toBeInTheDocument()
    expect(
      i18n.t("agentChat.composer.sourceContext.compactLabel")
    ).toBe("Files and images")
    expect(
      i18n.t("agentChat.strategy.options.learning.label")
    ).toBe("Give suggestions")
  })

  it("keeps Chinese toolbar copy understandable for first-time users", () => {
    const sourceContext = zhMessages.agentChat.composer.sourceContext
    const productContext = zhMessages.agentChat.composer.productContext
    const campaignContext = zhMessages.agentChat.composer.campaignContext
    const strategy = zhMessages.agentChat.strategy
    const shell = zhMessages.agentChat.shell

    expect(shell.brandTitle).toBe(copy.agentBrand)
    expect(shell.navigation.smartOps).toBe(copy.agentBrand)
    expect(shell.navigation.productBoard).toBe(copy.productAnalysis)
    expect(shell.brandTitle).not.toBe(copy.oldAgentBrand)
    expect(shell.navigation.productBoard).not.toBe(copy.oldProductBoard)
    expect(sourceContext.compactLabel).toBe(copy.filesAndImages)
    expect(productContext.noneCompactLabel).toBe(copy.product)
    expect(campaignContext.noneCompactLabel).toBe(copy.activity)
    expect(productContext.noneSelected).toBe(copy.joinProductReference)
    expect(campaignContext.noneSelected).toBe(copy.joinActivityReference)
    expect(strategy.options.learning.label).not.toBe(copy.thinkTogether)
    expect(productContext.noneCompactLabel).not.toBe(copy.unselectedProduct)
    expect(campaignContext.noneCompactLabel).not.toBe(copy.unselectedActivity)
  })

  it("keeps Chinese multi-agent mode copy aligned with ecommerce role-agent selection", () => {
    const runtime = zhMessages.agentChat.composer.runtime
    const processKinds = zhMessages.agentChat.thread.processKinds

    expect(runtime.roleMode).toBe(copy.multiAgentMode)
    expect(runtime.selectedRoles).toBe(copy.selectedAgentRoles)
    expect(runtime.roleModes.auto.label).toBe(copy.systemAutoAgents)
    expect(runtime.roleModes.single.label).toBe(copy.manualSingleAgent)
    expect(runtime.roleModes.manual.label).toBe(copy.manualMultipleAgents)
    expect(processKinds.single_agent.title).toBe(copy.singleAgentJudgment)
    expect(processKinds.multi_agent.title).toBe(copy.multiAgentJudgment)
    expect(runtime.roleMode).not.toBe(copy.oldReferenceMode)
    expect(runtime.selectedRoles).not.toBe(copy.oldSelectedReferences)
    expect(runtime.roleModes.auto.label).not.toBe(copy.oldAutoJudge)
    expect(runtime.roleModes.single.label).not.toBe(copy.oldFocusOneReference)
    expect(runtime.roleModes.manual.label).not.toBe(copy.oldMultiReference)
    expect(processKinds.single_agent.title).not.toBe(copy.oldFocusPerspective)
    expect(processKinds.multi_agent.title).not.toBe(copy.oldMultiReference)
    expect(processKinds.sanitized.multiReference).not.toBe(
      copy.oldMultiReference
    )
  })

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
    <MemoryRouter><AgentChatTestI18nProvider i18n={i18n}>{children}</AgentChatTestI18nProvider></MemoryRouter>
  )
}

function textFromCodes(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
