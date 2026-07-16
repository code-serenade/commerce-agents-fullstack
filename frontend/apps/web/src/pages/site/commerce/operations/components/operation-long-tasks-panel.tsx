import { useTranslation } from "react-i18next"
import { Button, NativeSelect } from "@workspace/ui-components"
import type { LongTaskResponse } from "@/api"
import {
  CommercePanel,
  CommerceTable,
  MutedText,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import {
  commerceStatusOptions,
  getCommerceEnumLabel,
} from "../../domain/enum-mappings"
import type { OperationsPageState } from "../use-operations-page"

export function LongTasksPanel({ page }: { page: OperationsPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.operations.panels.longTasks")}
      description={t("commerce.operations.panels.longTasksDesc")}
      actions={
        <Button size="sm" onClick={() => page.setDialog("long-task")}>
          {t("commerce.operations.panels.createLongTask")}
        </Button>
      }
    >
      <CommerceTable
        loading={page.loading}
        items={page.longTasks}
        emptyText={t("commerce.operations.panels.emptyLongTasks")}
        getRowId={(item) => item.id}
        columns={longTaskColumns(page, t)}
      />
    </CommercePanel>
  )
}

function longTaskColumns(
  page: OperationsPageState,
  t: CommerceTranslate
): CommerceColumn<LongTaskResponse>[] {
  return [
    {
      key: "task",
      header: t("commerce.operations.columns.task"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.title}</div>
          <MutedText>
            {getCommerceEnumLabel(t, "taskType", item.taskType)} /{" "}
            {item.currentStep || "-"}
          </MutedText>
        </div>
      ),
    },
    {
      key: "progress",
      header: t("commerce.operations.columns.progress"),
      render: (item) => `${item.progress ?? 0}/${item.totalSteps ?? 100}`,
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <NativeSelect
          value={item.status}
          options={longTaskStatusOptions(t)}
          onValueChange={(status) => page.updateLongTaskStatus({ task: item, status })}
        />
      ),
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      render: (item) => (
        <Button size="xs" variant="outline" onClick={() => page.cancelLongTask(item)}>
          {t("commerce.common.cancel")}
        </Button>
      ),
    },
  ]
}

function longTaskStatusOptions(t: CommerceTranslate) {
  return commerceStatusOptions(t, [
    "pending",
    "running",
    "completed",
    "failed",
    "cancelled",
  ])
}
