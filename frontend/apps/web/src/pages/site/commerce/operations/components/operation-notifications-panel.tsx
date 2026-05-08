import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import type { BackgroundEventResponse } from "@/api"
import {
  CommercePanel,
  CommerceTable,
  MutedText,
  StatusBadge,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { OperationsPageState } from "../use-operations-page"

export function NotificationsPanel({ page }: { page: OperationsPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.operations.panels.notifications")}
      description={t("commerce.operations.panels.notificationsDesc")}
      actions={
        <div className="flex flex-wrap gap-2">
          <Button size="sm" variant="outline" onClick={() => page.consumeAllEvents()}>
            {t("commerce.operations.panels.markAllRead")}
          </Button>
          <Button size="sm" onClick={() => page.setDialog("notification")}>
            {t("commerce.operations.panels.createNotification")}
          </Button>
        </div>
      }
    >
      <CommerceTable
        loading={page.loading}
        items={page.notifications}
        emptyText={t("commerce.operations.panels.emptyNotifications")}
        getRowId={(item) => item.id}
        columns={notificationColumns(page, t)}
      />
    </CommercePanel>
  )
}

function notificationColumns(
  page: OperationsPageState,
  t: CommerceTranslate
): CommerceColumn<BackgroundEventResponse>[] {
  return [
    {
      key: "title",
      header: t("commerce.operations.columns.notification"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.title}</div>
          <MutedText>{item.eventType}</MutedText>
        </div>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={item.consumed ? "success" : "warning"}>
          {item.consumed
            ? t("commerce.operations.states.read")
            : t("commerce.operations.states.unread")}
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
          disabled={item.consumed}
          onClick={() => page.consumeEvent(item)}
        >
          {t("commerce.common.markRead")}
        </Button>
      ),
    },
  ]
}
