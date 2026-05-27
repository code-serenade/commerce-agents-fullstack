import { Alert } from "@workspace/ui-components/stable/alert"
import { Button } from "@workspace/ui-components/stable/button"
import { useTranslation } from "react-i18next"

export interface AgentChatStatusProps {
  error: string | null
  loginRequired: boolean
  onLogin: () => void
}

export function AgentChatStatus({
  error,
  loginRequired,
  onLogin,
}: AgentChatStatusProps) {
  const { t } = useTranslation()

  if (!error && !loginRequired) return null

  return (
    <div className="mx-auto flex w-full max-w-6xl shrink-0 flex-col gap-2 px-4 pt-3 sm:px-6 sm:pt-4">
      {loginRequired ? (
        <div
          className="flex items-center justify-between gap-2 rounded-lg border border-border bg-card p-2.5 shadow-sm sm:p-3"
          data-agent-chat-status-card="true"
        >
          <div className="min-w-0 flex-1">
            <p className="text-sm/tight font-medium text-foreground">
              {t("agentChat.loginRequired.title")}
            </p>
            <p className="mt-1 line-clamp-2 text-xs/5 text-muted-foreground sm:text-sm">
              {t("agentChat.loginRequired.description")}
            </p>
          </div>
          <span className="shrink-0">
            <Button size="sm" onClick={onLogin}>
              {t("agentChat.loginRequired.action")}
            </Button>
          </span>
        </div>
      ) : null}
      {error ? (
        <div data-agent-chat-status-card="true">
          <Alert variant="error" description={error} />
        </div>
      ) : null}
    </div>
  )
}
