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
    <div className="mx-auto flex w-full max-w-6xl shrink-0 flex-col gap-2 px-4 pt-4 sm:px-6">
      {loginRequired ? (
        <div className="flex flex-col gap-2 rounded-lg border border-border bg-card p-3 shadow-sm sm:flex-row sm:items-center sm:justify-between">
          <div className="min-w-0">
            <p className="text-sm font-medium text-foreground">
              {t("agentChat.loginRequired.title")}
            </p>
            <p className="mt-1 text-sm text-muted-foreground">
              {t("agentChat.loginRequired.description")}
            </p>
          </div>
          <Button size="sm" onClick={onLogin}>
            {t("agentChat.loginRequired.action")}
          </Button>
        </div>
      ) : null}
      {error ? <Alert variant="error" description={error} /> : null}
    </div>
  )
}
