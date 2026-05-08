import { useTranslation } from "react-i18next"
import type { PlatformSyncStatusItem } from "@/api"
import {
  CommercePanel,
  CommerceTable,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { PlatformConnectPageState } from "../use-platform-connect-page"

export function PlatformSyncPanel({ page }: { page: PlatformConnectPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.platform.panels.sync")}
      description={t("commerce.platform.panels.syncDesc")}
    >
      <CommerceTable
        loading={page.loading}
        items={page.syncStatuses}
        emptyText={t("commerce.platform.empty.sync")}
        getRowId={(item) => item.platform}
        columns={syncColumns(t)}
      />
    </CommercePanel>
  )
}

function syncColumns(t: CommerceTranslate): CommerceColumn<PlatformSyncStatusItem>[] {
  return [
    {
      key: "platform",
      header: t("commerce.common.platform"),
      render: (item) => item.display,
    },
    {
      key: "status",
      header: t("commerce.platform.columns.sync"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.syncStatus || "idle")}>
          {item.syncStatus || "idle"}
        </StatusBadge>
      ),
    },
    {
      key: "date",
      header: t("commerce.platform.columns.latestMetric"),
      render: (item) => item.latestMetricDate || "-",
    },
  ]
}
