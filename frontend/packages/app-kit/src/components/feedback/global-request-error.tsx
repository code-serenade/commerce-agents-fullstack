import { useEffect } from "react"
import { X } from "lucide-react"

export interface GlobalRequestErrorProps {
  message: string | null
  onClose: () => void
  autoDismissMs?: number
  closeLabel?: string
}

export function GlobalRequestError({
  message,
  onClose,
  autoDismissMs = 5000,
  closeLabel = "Close error",
}: GlobalRequestErrorProps) {
  useEffect(() => {
    if (!message) {
      return
    }

    const timer = window.setTimeout(() => {
      onClose()
    }, autoDismissMs)

    return () => window.clearTimeout(timer)
  }, [autoDismissMs, message, onClose])

  if (!message) {
    return null
  }

  return (
    <div className="pointer-events-none fixed inset-x-0 top-4 z-[2500] flex justify-center px-4">
      <div className="pointer-events-auto inline-flex max-w-[min(90vw,48rem)] items-center gap-2 rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 text-sm text-destructive shadow-sm">
        <p className="truncate whitespace-nowrap">{message}</p>
        <button
          type="button"
          aria-label={closeLabel}
          className="inline-flex size-7 shrink-0 items-center justify-center rounded-md text-destructive/80 transition hover:bg-destructive/10 hover:text-destructive"
          onClick={onClose}
        >
          <X aria-hidden="true" className="size-4" />
        </button>
      </div>
    </div>
  )
}
