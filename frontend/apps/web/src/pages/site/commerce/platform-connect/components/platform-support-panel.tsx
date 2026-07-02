import { useTranslation } from "react-i18next"
import type { ChannelSpecResponse, PlatformSpecResponse } from "@/api"
import {
  CommercePanel,
  CommerceTable,
  MutedText,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { PlatformConnectPageState } from "../use-platform-connect-page"

export function PlatformSupportPanel({ page }: { page: PlatformConnectPageState }) {
  const { t } = useTranslation()

  return (
    <div className="grid gap-4 xl:grid-cols-2">
      <CommercePanel
        title={t("commerce.platform.panels.supportedPlatforms")}
        description={t("commerce.platform.panels.supportedPlatformsDesc")}
      >
        <CommerceTable
          loading={page.loading}
          items={page.supportedPlatforms}
          emptyText={t("commerce.platform.empty.supportedPlatforms")}
          getRowId={(item) => item.platform}
          columns={platformSpecColumns(t)}
        />
      </CommercePanel>
      <CommercePanel
        title={t("commerce.platform.panels.supportedChannels")}
        description={t("commerce.platform.panels.supportedChannelsDesc")}
      >
        <CommerceTable
          loading={page.loading}
          items={page.supportedChannels}
          emptyText={t("commerce.platform.empty.supportedChannels")}
          getRowId={(item) => item.channel}
          columns={channelSpecColumns(t)}
        />
      </CommercePanel>
    </div>
  )
}

function platformSpecColumns(
  t: CommerceTranslate
): CommerceColumn<PlatformSpecResponse>[] {
  return [
    {
      key: "platform",
      header: t("commerce.common.platform"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.display}</div>
          <MutedText>{item.platform}</MutedText>
        </div>
      ),
      minWidth: "12rem",
    },
    {
      key: "doc",
      header: t("commerce.common.document"),
      render: (item) => (
        <span className="block max-w-80 truncate" title={item.docUrl || undefined}>
          {item.docUrl || "-"}
        </span>
      ),
      minWidth: "18rem",
    },
  ]
}

function channelSpecColumns(
  t: CommerceTranslate
): CommerceColumn<ChannelSpecResponse>[] {
  return [
    {
      key: "channel",
      header: t("commerce.common.channel"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.display}</div>
          <MutedText>{item.channel}</MutedText>
        </div>
      ),
      minWidth: "12rem",
    },
    {
      key: "doc",
      header: t("commerce.common.document"),
      render: (item) => (
        <span className="block max-w-80 truncate" title={item.docUrl || undefined}>
          {item.docUrl || "-"}
        </span>
      ),
      minWidth: "18rem",
    },
  ]
}
