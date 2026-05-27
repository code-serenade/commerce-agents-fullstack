import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { FilesTable } from "./files-table"

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
})
