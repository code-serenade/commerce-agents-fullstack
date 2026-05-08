import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import type { DailyBriefingResponse } from "@/api"
import {
  CommercePanel,
  CommerceTable,
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { OperationsPageState } from "../use-operations-page"

export function DailyBriefingPanel({ page }: { page: OperationsPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.operations.panels.briefing")}
      description={
        page.dailySummary?.title || t("commerce.operations.panels.briefingDesc")
      }
      actions={
        <Button size="sm" onClick={() => page.setDialog("daily-briefing")}>
          {t("commerce.operations.panels.saveBriefing")}
        </Button>
      }
    >
      <CommerceTable
        loading={page.loading}
        items={page.dailyRecords}
        emptyText={t("commerce.operations.panels.emptyBriefing")}
        getRowId={(item) => item.id}
        columns={briefingColumns(page, t)}
      />
    </CommercePanel>
  )
}

function briefingColumns(
  page: OperationsPageState,
  t: CommerceTranslate
): CommerceColumn<DailyBriefingResponse>[] {
  return [
    {
      key: "title",
      header: t("commerce.operations.columns.briefing"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.title}</div>
          <MutedText>{item.summary || item.briefingDate}</MutedText>
        </div>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status || "ready")}>
          {item.status || "ready"}
        </StatusBadge>
      ),
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      render: (item) => (
        <Button size="xs" variant="outline" onClick={() => page.archiveDailyBriefing(item)}>
          {t("commerce.common.archive")}
        </Button>
      ),
    },
  ]
}
