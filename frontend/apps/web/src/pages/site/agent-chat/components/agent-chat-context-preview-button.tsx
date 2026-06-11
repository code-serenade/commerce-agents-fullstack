import { Eye, Loader2 } from "lucide-react"
import { useTranslation } from "react-i18next"

export function AgentChatContextPreviewButton({
  canSend,
  isLoggedIn,
  isOpen,
  isPreviewing,
  isStreaming,
  onOpenChange,
  onPreview,
}: {
  canSend: boolean
  isLoggedIn: boolean
  isOpen: boolean
  isPreviewing: boolean
  isStreaming: boolean
  onOpenChange: (open: boolean) => void
  onPreview: () => Promise<void>
}) {
  const { t } = useTranslation()
  const disabled = !canSend || !isLoggedIn || isStreaming
  const title = t("agentChat.composer.contextPreview.compactTitle")

  const handleClick = () => {
    if (!isOpen) onOpenChange(true)
    void onPreview()
  }

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={handleClick}
      className={`flex h-7 max-w-36 shrink-0 items-center gap-1.5 rounded-full border px-2.5 text-xs font-medium transition-colors ${
        isOpen
          ? "border-sky-400/70 bg-sky-50 text-sky-700 dark:bg-sky-500/15 dark:text-sky-200"
          : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground"
      } disabled:cursor-not-allowed disabled:opacity-45`}
      aria-expanded={isOpen}
      aria-label={title}
      title={title}
    >
      {isPreviewing ? (
        <Loader2 className="size-3.5 shrink-0 animate-spin" />
      ) : (
        <Eye className="size-3.5 shrink-0" />
      )}
      <span className="truncate">
        {t("agentChat.composer.contextPreview.compactLabel")}
      </span>
    </button>
  )
}
