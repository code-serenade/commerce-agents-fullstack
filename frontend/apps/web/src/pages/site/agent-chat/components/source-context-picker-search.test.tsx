import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { SourceContextPicker } from "./source-context-picker"
import {
  createIndexedLibraryItems,
  createSourceSelection,
  getIndexedLibraryTitle,
  sourcePickerTestTitles,
} from "./source-context-picker.test-data"

describe("SourceContextPicker search and paging", () => {
  it("filters current materials without clearing selected materials", async () => {
    const i18n = await createAgentChatTestI18n()
    const toggleSourceContext = vi.fn()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SourceContextPicker
          availableSources={[
            createSourceSelection({
              id: "sales-data",
              title: sourcePickerTestTitles.sales,
            }),
            createSourceSelection({
              id: "brief",
              title: sourcePickerTestTitles.brief,
            }),
          ]}
          disabled={false}
          onClearSourceContext={vi.fn()}
          onToggleSourceContext={toggleSourceContext}
          selectedSources={[
            createSourceSelection({
              id: "brief",
              title: sourcePickerTestTitles.brief,
            }),
          ]}
        />
      </AgentChatTestI18nProvider>
    )

    await userEvent.type(
      screen.getByPlaceholderText(
        i18n.t("agentChat.composer.sourceContext.currentSearch")
      ),
      "sales"
    )

    expect(screen.getByText(sourcePickerTestTitles.sales)).toBeInTheDocument()
    expect(
      screen.getAllByText(sourcePickerTestTitles.brief).length
    ).toBeGreaterThan(0)

    await userEvent.click(
      screen.getByRole("button", {
        name: new RegExp(sourcePickerTestTitles.brief, "i"),
      })
    )

    expect(toggleSourceContext).toHaveBeenCalledWith(
      expect.objectContaining({ id: "brief" })
    )
  })

  it("filters and expands upload file library files inside the upload file panel", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SourceContextPicker
          availableSources={[]}
          disabled={false}
          onClearSourceContext={vi.fn()}
          onToggleSourceContext={vi.fn()}
          selectedSources={[]}
          sourceLibrary={createIndexedLibraryItems(8)}
        />
      </AgentChatTestI18nProvider>
    )

    expect(screen.getByText(getIndexedLibraryTitle(0))).toBeInTheDocument()
    expect(screen.queryByText(getIndexedLibraryTitle(7))).not.toBeInTheDocument()

    await userEvent.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.composer.sourceContext.showMoreLibrary", {
          count: 2,
        }),
      })
    )

    expect(screen.getByText(getIndexedLibraryTitle(7))).toBeInTheDocument()

    await userEvent.type(
      screen.getByPlaceholderText(
        i18n.t("agentChat.composer.sourceContext.librarySearch")
      ),
      sourcePickerTestTitles.libraryQuery
    )

    expect(screen.getByText(getIndexedLibraryTitle(7))).toBeInTheDocument()
    expect(screen.queryByText(getIndexedLibraryTitle(0))).not.toBeInTheDocument()
  })
})
