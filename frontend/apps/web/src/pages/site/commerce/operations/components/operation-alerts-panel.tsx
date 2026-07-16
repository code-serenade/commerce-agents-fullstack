import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import type { AlertResponse } from "@/api"
import {
  CommercePanel,
  CommerceTable,
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import {
  getCommerceEnumLabel,
  getCommerceStatusLabel,
} from "../../domain/enum-mappings"
import type { OperationsPageState } from "../use-operations-page"

export function AlertsPanel({ page }: { page: OperationsPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.operations.panels.alerts")}
      description={t("commerce.operations.panels.activeAlerts", {
        count: page.alertSummary?.active ?? 0,
      })}
      actions={
        <div className="flex flex-wrap gap-2">
          <Button size="sm" variant="outline" onClick={() => page.dismissAllAlerts()}>
            {t("commerce.operations.panels.dismissAll")}
          </Button>
          <Button size="sm" onClick={() => page.setDialog("alert")}>
            {t("commerce.operations.panels.createAlert")}
          </Button>
        </div>
      }
    >
      <CommerceTable
        loading={page.loading}
        items={page.alerts}
        emptyText={t("commerce.operations.panels.emptyAlerts")}
        getRowId={(item) => item.id}
        columns={alertColumns(page, t)}
      />
    </CommercePanel>
  )
}

function alertColumns(
  page: OperationsPageState,
  t: CommerceTranslate
): CommerceColumn<AlertResponse>[] {
  return [
    {
      key: "message",
      header: t("commerce.operations.columns.alert"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">
            {getCommerceEnumLabel(t, "alertType", item.alertType)}
          </div>
          <MutedText>{item.message}</MutedText>
        </div>
      ),
    },
    {
      key: "severity",
      header: t("commerce.operations.columns.severity"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.severity)}>
          {getCommerceEnumLabel(t, "severity", item.severity)}
        </StatusBadge>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>
          {getCommerceStatusLabel(t, item.status)}
        </StatusBadge>
      ),
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      render: (item) => (
        <div className="flex flex-wrap gap-2">
          <Button size="xs" variant="outline" onClick={() => page.snoozeAlert(item)}>
            {t("commerce.common.snooze")}
          </Button>
          <Button size="xs" variant="outline" onClick={() => page.actAlert(item)}>
            {t("commerce.common.process")}
          </Button>
          <Button size="xs" variant="outline" onClick={() => page.dismissAlert(item)}>
            {t("commerce.common.close")}
          </Button>
        </div>
      ),
    },
  ]
}
