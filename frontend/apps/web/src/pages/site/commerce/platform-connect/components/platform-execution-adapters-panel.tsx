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
    },
    {
      key: "connection",
      header: t("commerce.platform.columns.connection"),
      render: (item) => (
        <div className="space-y-1">
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
    },
    {
      key: "adapter",
      header: t("commerce.platform.columns.adapter"),
      render: (item) => (
        <div className="space-y-1">
          <StatusBadge tone={toneForStatus(item.adapterStatus)}>
            {item.adapterStatus}
          </StatusBadge>
          <MutedText>{item.executionMode} / {item.riskLevel}</MutedText>
        </div>
      ),
    },
    {
      key: "readiness",
      header: t("commerce.platform.columns.readiness"),
      render: (item) => (
        <div className="max-w-80 space-y-1">
          <StatusBadge tone={toneForStatus(item.readiness)}>
            {item.readiness}
          </StatusBadge>
          <MutedText>{item.readinessMessage}</MutedText>
        </div>
      ),
    },
    {
      key: "params",
      header: t("commerce.fields.params"),
      render: (item) => (
        <div className="max-w-56">
          <MutedText>{item.requiredParams.join(", ")}</MutedText>
        </div>
      ),
    },
  ]
}
