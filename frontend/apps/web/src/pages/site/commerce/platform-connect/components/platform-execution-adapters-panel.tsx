import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import type {
  PlatformExecutionAdapterActionResponse,
  PlatformExecutionAdapterSpecResponse,
} from "@/api"
import {
  CommercePanel,
  CommerceTable,
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { PlatformConnectPageState } from "../use-platform-connect-page"

interface PlatformExecutionAdapterRow extends PlatformExecutionAdapterActionResponse {
  platform: string
  platformDisplay: string
  connected: boolean
  enabled: boolean
  connectionStatus: string
}

export function PlatformExecutionAdaptersPanel({
  page,
}: {
  page: PlatformConnectPageState
}) {
  const { t } = useTranslation()
  const rows = useMemo(
    () => flattenAdapters(page.platformExecutionAdapters),
    [page.platformExecutionAdapters]
  )

  return (
    <CommercePanel
      title={t("commerce.platform.panels.executionAdapters")}
      description={t("commerce.platform.panels.executionAdaptersDesc")}
    >
      <CommerceTable
        loading={page.loading}
        items={rows}
        emptyText={t("commerce.platform.empty.executionAdapters")}
        getRowId={(item) => `${item.platform}:${item.skillName}`}
        columns={executionAdapterColumns(t)}
      />
    </CommercePanel>
  )
}

function flattenAdapters(
  adapters: PlatformExecutionAdapterSpecResponse[]
): PlatformExecutionAdapterRow[] {
  return adapters.flatMap((adapter) =>
    adapter.actions.map((action) => ({
      ...action,
      platform: adapter.platform,
      platformDisplay: adapter.display,
      connected: adapter.connected,
      enabled: adapter.enabled,
      connectionStatus: adapter.connectionStatus,
    }))
  )
}

function executionAdapterColumns(
  t: CommerceTranslate
): CommerceColumn<PlatformExecutionAdapterRow>[] {
  return [
    {
      key: "platform",
      header: t("commerce.common.platform"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.platformDisplay}</div>
          <MutedText>{item.platform}</MutedText>
        </div>
      ),
      minWidth: "10rem",
    },
    {
      key: "action",
      header: t("commerce.fields.action"),
      render: (item) => (
        <div className="max-w-72 space-y-1">
          <div className="truncate font-medium">{item.display}</div>
          <MutedText>{item.skillName}</MutedText>
          <MutedText>{item.description}</MutedText>
        </div>
      ),
      minWidth: "20rem",
    },
    {
      key: "connection",
      header: t("commerce.platform.columns.connection"),
      render: (item) => (
        <div className="flex flex-wrap items-center gap-2">
          <StatusBadge tone={toneForStatus(item.connectionStatus)}>
            {item.connectionStatus}
          </StatusBadge>
          <MutedText>
            {item.connected && item.enabled
              ? t("commerce.platform.labels.connected")
              : t("commerce.platform.labels.notExecutable")}
          </MutedText>
        </div>
      ),
      minWidth: "13rem",
    },
    {
      key: "adapter",
      header: t("commerce.platform.columns.adapter"),
      render: (item) => (
        <div className="flex flex-wrap items-center gap-2">
          <StatusBadge tone={toneForStatus(item.adapterStatus)}>
            {item.adapterStatus}
          </StatusBadge>
          <MutedText>{item.executionMode} / {item.riskLevel}</MutedText>
        </div>
      ),
      minWidth: "15rem",
    },
    {
      key: "readiness",
      header: t("commerce.platform.columns.readiness"),
      render: (item) => (
        <div className="flex max-w-88 flex-wrap items-center gap-2">
          <StatusBadge tone={toneForStatus(item.readiness)}>
            {item.readiness}
          </StatusBadge>
          <MutedText>{item.readinessMessage}</MutedText>
        </div>
      ),
      minWidth: "24rem",
    },
    {
      key: "params",
      header: t("commerce.fields.params"),
      render: (item) => (
        <div className="max-w-104 space-y-1">
          <MutedText>
            {t("commerce.platform.labels.requiredParams")}:{" "}
            {item.requiredParams.join(", ") || "-"}
          </MutedText>
          <MutedText>
            {t("commerce.platform.labels.idempotencyFields")}:{" "}
            {item.idempotencyKeyFields.join(", ") || "-"}
          </MutedText>
          <MutedText>
            {item.canExecuteNow
              ? t("commerce.platform.labels.canExecuteNow")
              : t("commerce.platform.labels.dispatchPending")}
          </MutedText>
        </div>
      ),
      minWidth: "28rem",
    },
  ]
}
