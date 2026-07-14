import { useTranslation } from "react-i18next"
import { Link } from "react-router"
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
      <div className="inline-flex w-fit rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-3 py-1.5 text-xs font-medium text-foreground shadow-sm backdrop-blur-md">
        {t("agentChat.quota.loading")}
      </div>
    )
  }
  if (!quota) return null

  const isDisabled = quota.status === "disabled"
  const isExhausted = quota.questionRemaining <= 0
  const badgeVariant = isDisabled || isExhausted ? "outline" : "secondary"
  const isSchoolTeamBound =
    quota.isSchoolTeamBound ?? quota.questionLimit >= 150
  const baseQuestionLimit = quota.baseQuestionLimit ?? 100
  const schoolTeamBonusLimit =
    quota.schoolTeamBonusLimit ?? Math.max(quota.questionLimit - 100, 0)
  const quotaPolicyText = isSchoolTeamBound
    ? t("agentChat.quota.boundCompact", {
        limit: quota.questionLimit.toLocaleString(),
      })
    : t("agentChat.quota.unboundCompact", {
        base: baseQuestionLimit.toLocaleString(),
        bonus: schoolTeamBonusLimit.toLocaleString(),
        limit: (baseQuestionLimit + schoolTeamBonusLimit).toLocaleString(),
      })

  return (
    <div className="flex min-h-7 w-fit max-w-full flex-wrap items-center gap-x-1.5 gap-y-1 rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-3 py-1 text-[11px] font-medium text-foreground shadow-sm backdrop-blur-md">
      <Badge variant={badgeVariant}>
        {isDisabled
          ? t("agentChat.quota.disabledBadge")
          : isExhausted
            ? t("agentChat.quota.exhaustedBadge")
            : t("agentChat.quota.availableBadge")}
      </Badge>
      <span className="font-medium text-foreground">
        {t("agentChat.quota.compactUsage", {
          remaining: quota.questionRemaining.toLocaleString(),
          limit: quota.questionLimit.toLocaleString(),
        })}
      </span>
      <span className="hidden text-muted-foreground sm:inline">
        {quotaPolicyText}
      </span>
      {!isSchoolTeamBound ? (
        <Link
          to="/profile"
          className="cursor-pointer font-semibold text-primary underline-offset-4 hover:underline"
        >
          {t("agentChat.quota.unboundUpgrade")}
        </Link>
      ) : null}
      {isDisabled || isExhausted ? (
        <span className="w-full text-destructive sm:w-auto">
          {isDisabled
            ? t("agentChat.quota.disabledHint")
            : t("agentChat.quota.exhaustedHint")}
        </span>
      ) : null}
    </div>
  )
}
