import { useTranslation } from "react-i18next"
import { Badge } from "@workspace/ui-components"
import type { CurrentTeamUsageQuotaResponse } from "@/api"

export interface AgentChatQuotaStatusProps {
  isLoading: boolean
  quota: CurrentTeamUsageQuotaResponse | null
}

export function AgentChatQuotaStatus({
  isLoading,
  quota,
}: AgentChatQuotaStatusProps) {
  const { t } = useTranslation()

  if (isLoading) {
    return (
      <div className="border-b bg-muted/30 px-3 py-2 text-xs text-muted-foreground">
        {t("agentChat.quota.loading")}
      </div>
    )
  }
  if (!quota) return null

  const isDisabled = quota.status === "disabled"
  const isExhausted = quota.questionRemaining <= 0
  const badgeVariant = isDisabled || isExhausted ? "outline" : "secondary"

  return (
    <div className="flex flex-wrap items-center gap-2 border-b bg-muted/30 px-3 py-2 text-xs text-muted-foreground">
      <Badge variant={badgeVariant}>
        {isDisabled
          ? t("agentChat.quota.disabledBadge")
          : isExhausted
            ? t("agentChat.quota.exhaustedBadge")
            : t("agentChat.quota.availableBadge")}
      </Badge>
      <span className="font-medium text-foreground">{quota.policyText}</span>
      <span>
        {t("agentChat.quota.usage", {
          used: quota.questionUsed.toLocaleString(),
          remaining: quota.questionRemaining.toLocaleString(),
          limit: quota.questionLimit.toLocaleString(),
        })}
      </span>
      <span>
        {t("agentChat.quota.account", {
          account: quota.displayId || quota.displayName || quota.userId,
        })}
      </span>
      <span>
        {t("agentChat.quota.profile", {
          school: quota.schoolName,
          team: quota.teamName,
        })}
      </span>
      {isDisabled || isExhausted ? (
        <span className="text-destructive">
          {isDisabled
            ? t("agentChat.quota.disabledHint")
            : t("agentChat.quota.exhaustedHint")}
        </span>
      ) : null}
    </div>
  )
}
