import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { SourceContextPicker } from "./source-context-picker"

const selectedExtraTitle = "selected-extra.xlsx"
const selectedFileTitle = "selected-brief.pdf"
const selectedImageTitle = "selected-image.png"
const selectedHiddenTitle = "selected-hidden.docx"
const selectedMetadataOnlyTitle = "selected-metadata-only.csv"
const selectedPendingTitle = "selected-pending.docx"
const selectedFailedTitle = "selected-failed.txt"
const selectedUnsupportedTitle = "selected-unsupported.xls"

describe("SourceContextSelectionSummary", () => {
  it("confirms selected material deletion with target names and deselect guidance", async () => {
    const i18n = await createAgentChatTestI18n()
    const deleteSelectedSources = vi.fn()
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(true)

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SourceContextPicker
          availableSources={[]}
          disabled={false}
          onClearSourceContext={vi.fn()}
          onDeleteSelectedSources={deleteSelectedSources}
          onToggleSourceContext={vi.fn()}
          selectedSources={[
            {
              id: "file-selected",
              kind: "conversation_file",
              readable: true,
              ref: "conversation_file:file-selected",
              status: "uploaded",
              title: selectedFileTitle,
            },
          ]}
        />
      </AgentChatTestI18nProvider>
    )

    await userEvent.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.composer.sourceContext.deleteSelected"),
      })
    )

    expect(confirm).toHaveBeenCalledWith(
      expect.stringContaining(selectedFileTitle)
    )
    expect(confirm).toHaveBeenCalledWith(
      expect.stringContaining(
        i18n.t("agentChat.composer.sourceContext.clear")
      )
    )
    expect(deleteSelectedSources).toHaveBeenCalledTimes(1)

    confirm.mockRestore()
  })

  it("previews which selected materials will be read this turn", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SourceContextPicker
          availableSources={[]}
          disabled={false}
          onClearSourceContext={vi.fn()}
          onToggleSourceContext={vi.fn()}
          selectedSources={[
            {
              id: "file-selected",
              kind: "conversation_file",
              readable: true,
              ref: "conversation_file:file-selected",
              status: "uploaded",
              title: selectedFileTitle,
            },
            {
              id: "file-extra",
              kind: "conversation_file",
              readable: true,
              ref: "conversation_file:file-extra",
              status: "uploaded",
              title: selectedExtraTitle,
            },
            {
              id: "file-image",
              kind: "conversation_file",
              readable: true,
              ref: "conversation_file:file-image",
              status: "uploaded",
              title: selectedImageTitle,
            },
            {
              id: "file-hidden",
              kind: "conversation_file",
              readable: true,
              ref: "conversation_file:file-hidden",
              status: "uploaded",
              title: selectedHiddenTitle,
            },
          ]}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(
        i18n.t("agentChat.composer.sourceContext.selectedPreview", {
          hiddenText: i18n.t(
            "agentChat.composer.sourceContext.selectedPreviewHidden",
            { count: 1 }
          ),
          names: [
            selectedFileTitle,
            selectedExtraTitle,
            selectedImageTitle,
          ].join(" / "),
        })
      )
    ).toBeInTheDocument()
    expect(
      screen.getAllByText(selectedFileTitle).length
    ).toBeGreaterThanOrEqual(1)
    expect(
      screen.getByText(
        i18n.t("agentChat.composer.sourceContext.selectedPreviewHidden", {
          count: 1,
        }),
        { exact: false }
      )
    ).toBeInTheDocument()
  })

  it("summarizes read status for selected materials before this turn sends", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SourceContextPicker
          availableSources={[]}
          disabled={false}
          onClearSourceContext={vi.fn()}
          onToggleSourceContext={vi.fn()}
          selectedSources={[
            {
              id: "file-readable",
              kind: "conversation_file",
              readable: true,
              ref: "conversation_file:file-readable",
              status: "uploaded",
              title: selectedFileTitle,
            },
            {
              id: "file-image",
              kind: "conversation_file",
              readable: false,
              ref: "conversation_file:file-image",
              status: "uploaded",
              title: selectedImageTitle,
              visionReady: true,
            },
            {
              id: "file-pending",
              kind: "conversation_file",
              readable: false,
              ref: "conversation_file:file-pending",
              status: "pending",
              title: selectedPendingTitle,
            },
            {
              id: "file-failed",
              kind: "conversation_file",
              readable: false,
              ref: "conversation_file:file-failed",
              status: "failed",
              title: selectedFailedTitle,
            },
            {
              id: "file-unsupported",
              kind: "conversation_file",
              readable: false,
              ref: "conversation_file:file-unsupported",
              status: "unsupported",
              title: selectedUnsupportedTitle,
            },
            {
              id: "file-metadata",
              kind: "conversation_file",
              readable: false,
              ref: "conversation_file:file-metadata",
              status: "uploaded",
              title: selectedMetadataOnlyTitle,
            },
          ]}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(
        i18n.t("agentChat.composer.sourceContext.selectedReadinessLabel")
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        i18n.t("agentChat.composer.sourceContext.selectedReadiness.readable", {
          count: 2,
        })
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        i18n.t("agentChat.composer.sourceContext.selectedReadiness.pending", {
          count: 1,
        })
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        i18n.t("agentChat.composer.sourceContext.selectedReadiness.failed", {
          count: 1,
        })
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        i18n.t(
          "agentChat.composer.sourceContext.selectedReadiness.unsupported",
          { count: 1 }
        )
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        i18n.t(
          "agentChat.composer.sourceContext.selectedReadiness.metadataOnly",
          { count: 1 }
        )
      )
    ).toBeInTheDocument()
  })
})
