import { fireEvent, render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { Dialog, DialogContent, DialogTitle } from "@workspace/ui-components"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { CommerceDetailDrawer } from "./commerce-detail-drawer"

describe("CommerceDetailDrawer", () => {
  it("renders an accessible shared detail surface", async () => {
    const i18n = await createAgentChatTestI18n()
    const title = i18n.t("commerce.products.panels.detail")
    const description = i18n.t("commerce.products.panels.detailDesc")
    const closeLabel = i18n.t("commerce.common.close")

    render(
      <CommerceDetailDrawer
        open
        closeLabel={closeLabel}
        description={description}
        onClose={vi.fn()}
        title={title}
      >
        <div>Drawer content</div>
      </CommerceDetailDrawer>
    )

    expect(
      screen.getByRole("dialog", { name: title })
    ).toHaveTextContent("Drawer content")
    expect(screen.getByText(description)).toBeInTheDocument()
    expect(
      screen.getByRole("button", { name: closeLabel })
    ).toBeInTheDocument()
  })

  it("closes from the button, backdrop, and Escape key", async () => {
    const i18n = await createAgentChatTestI18n()
    const title = i18n.t("commerce.products.panels.detail")
    const closeLabel = i18n.t("commerce.common.close")
    const onClose = vi.fn()

    render(
      <CommerceDetailDrawer
        open
        closeLabel={closeLabel}
        onClose={onClose}
        title={title}
      >
        <div>Drawer content</div>
      </CommerceDetailDrawer>
    )

    fireEvent.click(screen.getByRole("button", { name: closeLabel }))
    fireEvent.pointerDown(document.querySelector("[role='presentation']")!)
    fireEvent.keyDown(window, { key: "Escape" })

    expect(onClose).toHaveBeenCalledTimes(3)
  })

  it("stays below modal dialogs in the overlay stack", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <>
        <CommerceDetailDrawer
          open
          closeLabel={i18n.t("commerce.common.close")}
          onClose={vi.fn()}
          title={i18n.t("commerce.products.panels.detail")}
        >
          <div>Drawer content</div>
        </CommerceDetailDrawer>
        <Dialog open onOpenChange={vi.fn()}>
          <DialogContent>
            <DialogTitle>Nested action</DialogTitle>
          </DialogContent>
        </Dialog>
      </>
    )

    expect(document.querySelector("[role='presentation']")).toHaveClass("z-40")
    expect(document.querySelector("[data-slot='dialog-overlay']")).toHaveClass(
      "z-50"
    )
    expect(document.querySelector("[data-slot='dialog-content']")).toHaveClass(
      "z-50"
    )
  })
})
