import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import type {
  ChannelConnectionResponse,
  PlatformConnectionResponse,
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

export function PlatformConnectionsPanel({ page }: { page: PlatformConnectPageState }) {
  const { t } = useTranslation()

  return (
    <div className="grid gap-4 xl:grid-cols-2">
      <CommercePanel
        title={t("commerce.platform.panels.platformConnections")}
        description={t("commerce.platform.panels.platformConnectionsDesc")}
        actions={
          <Button size="sm" onClick={() => page.setDialog("platform")}>
            {t("commerce.platform.panels.savePlatform")}
          </Button>
        }
      >
        <CommerceTable
          loading={page.loading}
          items={page.platformConnections}
          emptyText={t("commerce.platform.empty.platformConnections")}
          getRowId={(item) => item.id}
          columns={platformConnectionColumns(page, t)}
        />
      </CommercePanel>
      <CommercePanel
        title={t("commerce.platform.panels.channelConnections")}
        description={t("commerce.platform.panels.channelConnectionsDesc")}
        actions={
          <Button size="sm" onClick={() => page.setDialog("channel")}>
            {t("commerce.platform.panels.saveChannel")}
          </Button>
        }
      >
        <CommerceTable
          loading={page.loading}
          items={page.channelConnections}
          emptyText={t("commerce.platform.empty.channelConnections")}
          getRowId={(item) => item.id}
          columns={channelConnectionColumns(page, t)}
        />
      </CommercePanel>
    </div>
  )
}

function platformConnectionColumns(
  page: PlatformConnectPageState,
  t: CommerceTranslate
): CommerceColumn<PlatformConnectionResponse>[] {
  return [
    {
      key: "platform",
      header: t("commerce.common.platform"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.display || item.platform}</div>
          <MutedText>{item.shopName || item.shopId || "-"}</MutedText>
        </div>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>
          {item.enabled ? item.status : "disabled"}
        </StatusBadge>
      ),
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      render: (item) => (
        <Button
          size="xs"
          variant="outline"
          onClick={() => page.disablePlatform(item.id)}
        >
          {t("commerce.common.disable")}
        </Button>
      ),
    },
  ]
}

function channelConnectionColumns(
  page: PlatformConnectPageState,
  t: CommerceTranslate
): CommerceColumn<ChannelConnectionResponse>[] {
  return [
    {
      key: "channel",
      header: t("commerce.common.channel"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.display || item.channel}</div>
          <MutedText>{item.lastError || "-"}</MutedText>
        </div>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status || (item.enabled ? "active" : "disabled"))}>
          {item.enabled ? item.status || "active" : "disabled"}
        </StatusBadge>
      ),
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      render: (item) => (
        <Button
          size="xs"
          variant="outline"
          onClick={() => page.disableChannel(item.id)}
        >
          {t("commerce.common.disable")}
        </Button>
      ),
    },
  ]
}
