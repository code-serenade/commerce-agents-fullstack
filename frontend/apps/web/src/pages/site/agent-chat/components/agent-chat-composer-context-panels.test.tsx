import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import type { ReactNode } from "react"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { AgentChatComposerPanels } from "./agent-chat-composer-panels"
import { AgentChatComposerToolbar } from "./agent-chat-composer-toolbar"
import {
  createLibraryItem,
  createSourceSelection,
  sourcePickerTestTitles,
} from "./source-context-picker.test-data"
import {
  basePanelProps,
  baseToolbarProps,
  campaign,
  product,
} from "./agent-chat-composer-test-fixtures"

describe("AgentChatComposer context panels", () => {
  it("does not expose the context preview action in the composer toolbar", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerToolbar {...baseToolbarProps()} />
      </TestProviders>
    )

    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.composer.contextPreview.compactTitle"),
      })
    ).not.toBeInTheDocument()
  })

  it("keeps product and campaign entry points visible before context is selected", async () => {
    const i18n = await createAgentChatTestI18n()
    const { rerender } = render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerToolbar {...baseToolbarProps()} />
      </TestProviders>
    )

    expect(
      screen.getByRole("button", {
        name: new RegExp(
          i18n.t("agentChat.composer.productContext.compactTitle")
        ),
      })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", {
        name: new RegExp(
          i18n.t("agentChat.composer.campaignContext.compactTitle")
        ),
      })
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.composer.productContext.compactLabel"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.composer.campaignContext.compactLabel"))
    ).toBeInTheDocument()

    rerender(
      <TestProviders i18n={i18n}>
        <AgentChatComposerToolbar
          {...baseToolbarProps()}
          availableProducts={[product]}
          selectedCampaigns={[campaign]}
        />
      </TestProviders>
    )

    expect(
      screen.getByRole("button", {
        name: new RegExp(
          i18n.t("agentChat.composer.productContext.compactTitle")
        ),
      })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", {
        name: new RegExp(
          i18n.t("agentChat.composer.campaignContext.compactTitle")
        ),
      })
    ).toBeInTheDocument()
  })

  it("opens upload file library, toggles sources, and closes the panel", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()
    const source = createSourceSelection({
      id: "sales-data",
      title: sourcePickerTestTitles.sales,
    })
    const libraryFile = createLibraryItem({
      id: "library-1",
      objectKey: "docs/library-1.pdf",
      title: sourcePickerTestTitles.library,
    })
    const onToggleSourcePanel = vi.fn()
    const onClosePanels = vi.fn()
    const onToggleSourceContext = vi.fn()
    const onAttachExistingSource = vi.fn()
    const onUploadClick = vi.fn()

    const { rerender } = render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerToolbar
          {...baseToolbarProps()}
          availableSources={[source]}
          onToggleSourcePanel={onToggleSourcePanel}
        />
      </TestProviders>
    )

    await user.click(
      screen.getByRole("button", {
        name: new RegExp(
          i18n.t("agentChat.composer.sourceContext.compactTitle")
        ),
      })
    )
    expect(onToggleSourcePanel).toHaveBeenCalledWith(true)

    rerender(
      <TestProviders i18n={i18n}>
        <AgentChatComposerToolbar
          {...baseToolbarProps()}
          availableSources={[source]}
          sourceContextOpen
        />
        <AgentChatComposerPanels
          {...basePanelProps()}
          availableSources={[source]}
          onAttachExistingSource={onAttachExistingSource}
          onClosePanels={onClosePanels}
          onToggleSourceContext={onToggleSourceContext}
          onUploadClick={onUploadClick}
          sourceContextOpen
          sourceLibrary={[libraryFile]}
        />
      </TestProviders>
    )

    expect(
      screen.getByText(i18n.t("agentChat.composer.sourceContext.panelTitle"))
    ).toBeInTheDocument()
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.composer.sourceContext.upload"),
      })
    )
    expect(onUploadClick).toHaveBeenCalledTimes(1)
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.composer.sourceContext.manageLibrary"),
      })
    ).not.toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: /sales-data\.csv/i }))
    expect(onToggleSourceContext).toHaveBeenCalledWith(source)
    await user.click(screen.getByRole("button", { name: /file-library\.pdf/i }))
    expect(onAttachExistingSource).toHaveBeenCalledWith("library-1")
    await user.click(screen.getByRole("button", { name: "actions.close" }))
    expect(onClosePanels).toHaveBeenCalledTimes(1)
  })

  it("toggles product and campaign context selections", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()
    const onToggleProductContext = vi.fn()
    const onToggleCampaignContext = vi.fn()
    const { rerender } = render(
      <TestProviders i18n={i18n}>
        <AgentChatComposerPanels
          {...basePanelProps()}
          availableProducts={[product]}
          onToggleProductContext={onToggleProductContext}
          productContextOpen
        />
      </TestProviders>
    )

    await user.click(screen.getByRole("button", { name: /Temperature thermos/i }))
    expect(onToggleProductContext).toHaveBeenCalledWith(product)

    rerender(
      <TestProviders i18n={i18n}>
        <AgentChatComposerPanels
          {...basePanelProps()}
          availableCampaigns={[campaign]}
          campaignContextOpen
          onToggleCampaignContext={onToggleCampaignContext}
        />
      </TestProviders>
    )

    await user.click(
      await screen.findByRole("button", {
        name: /7-day validation campaign/i,
      })
    )
    expect(onToggleCampaignContext).toHaveBeenCalledWith(campaign)
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
