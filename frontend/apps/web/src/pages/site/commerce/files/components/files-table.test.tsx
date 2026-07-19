import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { FilesTable } from "./files-table"
import { file } from "../domain/test-helpers"

describe("FilesTable", () => {
  it("uses the file manager upload action when the library is empty", async () => {
    const i18n = await createAgentChatTestI18n()
    const uploadEmpty = vi.fn()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <FilesTable
          dataFileCount={0}
          files={[]}
          formatFileSize={() => "-"}
          getFileConversationUseState={() => "available"}
          getFileName={() => ""}
          hasActiveFilters={false}
          loading={false}
          onClearFilters={vi.fn()}
          onOpenFileDetail={vi.fn()}
          onToggleCurrentTurn={vi.fn()}
          onToggleFileSelection={vi.fn()}
          onUploadEmpty={uploadEmpty}
          selectedFileIds={new Set()}
          turnActionsEnabled
          usingFileIds={new Set()}
        />
      </AgentChatTestI18nProvider>
    )

    await userEvent.click(
      screen.getByRole("button", {
        name: i18n.t("commerce.files.emptyAction"),
      })
    )

    expect(uploadEmpty).toHaveBeenCalledTimes(1)
  })

  it("keeps current-turn actions disabled when chat context is unavailable", async () => {
    const i18n = await createAgentChatTestI18n()
    const toggleCurrentTurn = vi.fn()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <FilesTable
          canUploadEmpty={false}
          dataFileCount={1}
          files={[file({ id: "material-1", fileName: "brief.pdf" })]}
          formatFileSize={() => "-"}
          getFileConversationUseState={() => "available"}
          getFileName={(item) => item.fileName ?? ""}
          hasActiveFilters={false}
          loading={false}
          onClearFilters={vi.fn()}
          onOpenFileDetail={vi.fn()}
          onToggleCurrentTurn={toggleCurrentTurn}
          onToggleFileSelection={vi.fn()}
          selectedFileIds={new Set()}
          turnActionsEnabled={false}
          usingFileIds={new Set()}
        />
      </AgentChatTestI18nProvider>
    )

    const useButtons = screen.getAllByRole("button", {
      name: i18n.t("commerce.files.table.useState.available"),
    })

    expect(useButtons).toHaveLength(2)
    useButtons.forEach((button) => expect(button).toBeDisabled())
    await userEvent.click(useButtons[0])
    expect(toggleCurrentTurn).not.toHaveBeenCalled()
  })
})
