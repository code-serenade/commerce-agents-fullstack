import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { file } from "../domain/test-helpers"
import { FilesDetailDrawer } from "./files-detail-drawer"

describe("FilesDetailDrawer", () => {
  it("renders file details in the shared commerce drawer", async () => {
    const i18n = await createAgentChatTestI18n()
    const title = i18n.t("commerce.files.detailDialog.title")

    render(
      <FilesDetailDrawer
        file={file({ fileName: "catalog.pdf" })}
        onOpenChange={vi.fn()}
        open
        t={i18n.t}
        useState="available"
      />
    )

    expect(screen.getByRole("dialog", { name: title })).toHaveTextContent(
      "catalog.pdf"
    )
    expect(
      document.querySelector("[data-slot='dialog-content']")
    ).not.toBeInTheDocument()
  })

  it("closes from the drawer close button", async () => {
    const i18n = await createAgentChatTestI18n()
    const onOpenChange = vi.fn()

    render(
      <FilesDetailDrawer
        file={file({})}
        onOpenChange={onOpenChange}
        open
        t={i18n.t}
        useState={null}
      />
    )

    await userEvent.click(
      screen.getByRole("button", {
        name: i18n.t("commerce.files.detailDialog.close"),
      })
    )

    expect(onOpenChange).toHaveBeenCalledWith(false)
  })
})
