import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { SourceContextPicker } from "./source-context-picker"
import {
  createLibraryItem,
  createSourceSelection,
  sourcePickerTestTitles,
} from "./source-context-picker.test-data"

describe("SourceContextPicker", () => {
  it("shows current conversation material read states", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SourceContextPicker
          availableSources={[
            createSourceSelection({
              id: "file-pending",
              readable: false,
              status: "pending",
              title: sourcePickerTestTitles.pending,
              visionReady: false,
            }),
            createSourceSelection({
              id: "file-unsupported",
              readable: false,
              status: "unsupported",
              title: sourcePickerTestTitles.unsupported,
              visionReady: false,
            }),
          ]}
          disabled={false}
          onClearSourceContext={vi.fn()}
          onToggleSourceContext={vi.fn()}
          selectedSources={[]}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(
        i18n.t("agentChat.composer.sourceContext.sourceStatus.pending")
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        i18n.t("agentChat.composer.sourceContext.sourceStatus.unsupported")
      )
    ).toBeInTheDocument()
  })

  it("lets users attach a library file before selecting it for this turn", async () => {
    const i18n = await createAgentChatTestI18n()
    const attachExistingSource = vi.fn()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SourceContextPicker
          availableSources={[]}
          disabled={false}
          onClearSourceContext={vi.fn()}
          onToggleSourceContext={vi.fn()}
          onAttachExistingSource={attachExistingSource}
          selectedSources={[]}
          sourceLibrary={[
            createLibraryItem({
              attached: false,
              id: "file-library",
              objectKey: "docs/file-library.pdf",
              title: sourcePickerTestTitles.library,
            }),
          ]}
        />
      </AgentChatTestI18nProvider>
    )

    await userEvent.click(
      screen.getByRole("button", { name: /file-library\.pdf/i })
    )

    expect(attachExistingSource).toHaveBeenCalledWith("file-library")
    expect(
      screen.getByText(
        i18n.t("agentChat.composer.sourceContext.librarySection")
      )
    ).toBeInTheDocument()
  })

  it("opens the materials manager from the upload materials panel", async () => {
    const i18n = await createAgentChatTestI18n()
    const manageSources = vi.fn()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SourceContextPicker
          availableSources={[]}
          disabled={false}
          onClearSourceContext={vi.fn()}
          onManageSources={manageSources}
          onToggleSourceContext={vi.fn()}
          selectedSources={[]}
        />
      </AgentChatTestI18nProvider>
    )

    await userEvent.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.composer.sourceContext.manageLibrary"),
      })
    )

    expect(manageSources).toHaveBeenCalledTimes(1)
  })

  it("shows a library manager action when recent library files are empty", async () => {
    const i18n = await createAgentChatTestI18n()
    const manageSources = vi.fn()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SourceContextPicker
          availableSources={[]}
          disabled={false}
          onClearSourceContext={vi.fn()}
          onManageSources={manageSources}
          onToggleSourceContext={vi.fn()}
          selectedSources={[]}
          sourceLibrary={[]}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(
        i18n.t("agentChat.composer.sourceContext.libraryEmpty")
      )
    ).toBeInTheDocument()

    await userEvent.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.composer.sourceContext.libraryEmptyAction"),
      })
    )

    expect(manageSources).toHaveBeenCalledTimes(1)
  })

  it("does not reattach files already in the current conversation", async () => {
    const i18n = await createAgentChatTestI18n()
    const attachExistingSource = vi.fn()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SourceContextPicker
          availableSources={[]}
          disabled={false}
          onClearSourceContext={vi.fn()}
          onToggleSourceContext={vi.fn()}
          onAttachExistingSource={attachExistingSource}
          selectedSources={[]}
          sourceLibrary={[
            createLibraryItem({
              attached: true,
              id: "file-attached",
              objectKey: "docs/file-attached.pdf",
              title: sourcePickerTestTitles.attached,
            }),
          ]}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByRole("button", { name: /file-attached\.pdf/i })
    ).toBeDisabled()
    expect(attachExistingSource).not.toHaveBeenCalled()
  })
})
