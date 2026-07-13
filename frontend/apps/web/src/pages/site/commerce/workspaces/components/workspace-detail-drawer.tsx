import type { ReactNode } from "react"
import { useEffect, useId } from "react"
import { createPortal } from "react-dom"
import { X } from "lucide-react"
import { Button } from "@workspace/ui-components"

export function WorkspaceDetailDrawer({
  children,
  description,
  onClose,
  open,
  title,
}: {
  children: ReactNode
  description?: ReactNode
  onClose: () => void
  open: boolean
  title: ReactNode
}) {
  const titleId = useId()
  const descriptionId = useId()

  useEffect(() => {
    if (!open) return

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose()
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [onClose, open])

  if (!open || typeof document === "undefined") return null

  return createPortal(
    <div
      className="fixed inset-0 z-[1000] flex justify-end bg-background/45 backdrop-blur-[2px]"
      onPointerDown={onClose}
      role="presentation"
    >
      <section
        aria-describedby={description ? descriptionId : undefined}
        aria-labelledby={titleId}
        aria-modal="true"
        className="flex h-dvh w-[min(44rem,100%)] max-w-full min-w-0 flex-col border-l border-(--app-border) bg-background shadow-2xl"
        onPointerDown={(event) => event.stopPropagation()}
        role="dialog"
      >
        <header className="flex items-start justify-between gap-3 border-b border-(--app-border) px-5 py-4">
          <div className="min-w-0">
            <h2 className="truncate text-base font-semibold" id={titleId}>
              {title}
            </h2>
            {description ? (
              <p
                className="mt-1 text-sm text-muted-foreground"
                id={descriptionId}
              >
                {description}
              </p>
            ) : null}
          </div>
          <Button
            aria-label="Close"
            size="sm"
            type="button"
            variant="ghost"
            onClick={onClose}
          >
            <X className="size-4" />
          </Button>
        </header>
        <div className="min-h-0 flex-1 overflow-auto p-5">{children}</div>
      </section>
    </div>,
    document.body
  )
}
