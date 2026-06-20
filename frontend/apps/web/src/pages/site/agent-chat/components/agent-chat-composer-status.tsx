import type { ReactNode } from "react"
import { CircleAlert, Loader2, LockKeyhole, RotateCcw } from "lucide-react"
import { useTranslation } from "react-i18next"

export function AgentChatComposerStatus({
  canRetry = false,
  error,
  isLoggedIn,
  isStreaming,
  onRetry,
}: {
  canRetry?: boolean
  error: string | null
  isLoggedIn: boolean
  isStreaming: boolean
  onRetry?: () => void
}) {
  const { t } = useTranslation()

  if (isStreaming) {
    return (
      <ComposerStatusFrame tone="active">
        <Loader2 className="size-3.5 animate-spin" />
        <span>{t("agentChat.composer.status.sending")}</span>
      </ComposerStatusFrame>
    )
  }

  if (error) {
    return (
      <ComposerStatusFrame tone="error">
        <CircleAlert className="size-3.5" />
        <span className="min-w-0 truncate">{error}</span>
        {onRetry ? (
          <button
            type="button"
            disabled={!canRetry}
            onClick={onRetry}
            className="ml-auto inline-flex h-7 shrink-0 items-center gap-1 rounded-md border border-destructive/25 bg-background px-2 text-[11px] font-medium text-destructive transition hover:bg-destructive/10 disabled:cursor-not-allowed disabled:opacity-45"
          >
            <RotateCcw className="size-3" />
            {t("agentChat.composer.status.retry")}
          </button>
        ) : null}
      </ComposerStatusFrame>
    )
  }

  if (!isLoggedIn) {
    return (
      <ComposerStatusFrame tone="muted">
        <LockKeyhole className="size-3.5" />
        <span>{t("agentChat.loginRequired.inline")}</span>
      </ComposerStatusFrame>
    )
  }

  return null
}

function ComposerStatusFrame({
  children,
  tone,
}: {
  children: ReactNode
  tone: "active" | "error" | "muted"
}) {
  return (
    <div
      className={[
        "flex min-w-0 items-center gap-2 rounded-md px-2.5 py-1.5 text-xs",
        tone === "active"
          ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
          : tone === "error"
            ? "bg-destructive/10 text-destructive"
            : "bg-muted text-muted-foreground",
      ].join(" ")}
    >
      {children}
    </div>
  )
}
