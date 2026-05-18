import { useTranslation } from "react-i18next"
import {
  CommercePanel,
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { PlatformConnectPageState } from "../use-platform-connect-page"

export function PlatformExternalSearchPanel({
  page,
}: {
  page: PlatformConnectPageState
}) {
  const { t } = useTranslation()
  const status = page.externalSearchStatus

  return (
    <CommercePanel
      title={t("commerce.platform.panels.externalSearch")}
      description={t("commerce.platform.panels.externalSearchDesc")}
    >
      {status ? (
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          <SearchStatusItem
            label={t("commerce.platform.externalSearch.status")}
            value={status.status}
            tone={toneForStatus(status.status)}
          />
          <SearchStatusItem
            label={t("commerce.platform.externalSearch.provider")}
            value={status.provider || t("commerce.common.none")}
          />
          <SearchStatusItem
            label={t("commerce.platform.externalSearch.enabled")}
            value={
              status.enabled
                ? t("commerce.approvals.labels.yes")
                : t("commerce.approvals.labels.no")
            }
          />
          <SearchStatusItem
            label={t("commerce.platform.externalSearch.configured")}
            value={
              status.configured
                ? t("commerce.approvals.labels.yes")
                : t("commerce.approvals.labels.no")
            }
          />
        </div>
      ) : (
        <div className="rounded-lg border border-dashed border-(--app-border) p-6 text-sm text-muted-foreground">
          {page.loading
            ? t("commerce.common.loading")
            : t("commerce.platform.empty.externalSearch")}
        </div>
      )}
      {status?.requiredConfig ? (
        <div className="mt-3 rounded-lg border border-(--app-border) bg-muted/30 p-3">
          <MutedText>{status.requiredConfig}</MutedText>
        </div>
      ) : null}
    </CommercePanel>
  )
}

function SearchStatusItem({
  label,
  tone = "default",
  value,
}: {
  label: string
  tone?: Parameters<typeof StatusBadge>[0]["tone"]
  value: string
}) {
  return (
    <div className="min-w-0 rounded-lg border border-(--app-border) bg-background p-3">
      <MutedText>{label}</MutedText>
      <div className="mt-2">
        <StatusBadge tone={tone}>{value}</StatusBadge>
      </div>
    </div>
  )
}
