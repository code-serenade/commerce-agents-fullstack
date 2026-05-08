import { Button } from "@workspace/ui-components"
import type { ExecutionQueueResponse } from "@/api"
import {
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { WorkspacesPageState } from "../use-workspaces-page"

export function createExecutionColumns(
  page: WorkspacesPageState,
  t: CommerceTranslate
): CommerceColumn<ExecutionQueueResponse>[] {
  return [
    {
      key: "action",
      header: t("commerce.workspaces.columns.suggestion"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.actionType}</div>
          <MutedText>{item.description}</MutedText>
        </div>
      ),
    },
    {
      key: "platform",
      header: t("commerce.common.platform"),
      render: (item) => item.platform || "-",
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>{item.status}</StatusBadge>
      ),
    },
    {
      key: "actions",
      header: t("commerce.common.decision"),
      render: (item) => (
        <div className="flex flex-wrap gap-2">
          <Button
            size="xs"
            variant="outline"
            disabled={item.status !== "pending"}
            onClick={() => page.approveExecution(item)}
          >
            {t("commerce.common.approve")}
          </Button>
          <Button
            size="xs"
            variant="outline"
            disabled={item.status !== "pending"}
            onClick={() => page.rejectExecution(item)}
          >
            {t("commerce.common.reject")}
          </Button>
        </div>
      ),
    },
  ]
}
