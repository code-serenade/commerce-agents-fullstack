import type { MetricCardsItem } from "@workspace/app-kit"
import type {
  ExecutionQueueResponse,
  WorkspaceResponse,
  WorkspaceRunResponse,
  WorkspaceTaskResponse,
} from "@/api"
import type { CommerceTranslate } from "../components/commerce-types"

export function buildWorkspaceMetricCards(
  t: CommerceTranslate,
  data: {
    executions: ExecutionQueueResponse[]
    runs: WorkspaceRunResponse[]
    tasks: WorkspaceTaskResponse[]
    workspaces: WorkspaceResponse[]
  }
): MetricCardsItem[] {
  return [
    {
      key: "workspaces",
      label: t("commerce.workspaces.metrics.workspaces"),
      value: data.workspaces.length,
      tail: t("commerce.workspaces.metrics.workspacesTail"),
      variant: "accent",
    },
    {
      key: "tasks",
      label: t("commerce.workspaces.metrics.tasks"),
      value: data.tasks.length,
      tail: t("commerce.common.countDone", {
        count: data.tasks.filter((item) => item.status === "done").length,
      }),
      variant: "success",
    },
    {
      key: "runs",
      label: t("commerce.workspaces.metrics.runs"),
      value: data.runs.length,
      tail: t("commerce.workspaces.metrics.runsTail"),
      variant: "default",
    },
    {
      key: "executions",
      label: t("commerce.workspaces.metrics.executions"),
      value: data.executions.length,
      tail: t("commerce.common.countPending", {
        count: data.executions.filter((item) => item.status === "pending")
          .length,
      }),
      variant: "warning",
    },
  ]
}
