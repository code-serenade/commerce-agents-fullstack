import { Button, NativeSelect } from "@workspace/ui-components"
import type {
  WorkspaceResponse,
  WorkspaceRunResponse,
  WorkspaceRunTaskResponse,
  WorkspaceTaskResponse,
} from "@/api"
import {
  IdText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import {
  getCommerceEnumLabel,
  getCommerceStatusLabel,
  getWorkspaceTypeLabel,
  workspaceRunStatusOptions,
  workspaceRunTaskStatusOptions,
  workspaceTaskStatusOptions,
} from "../../domain/enum-mappings"
import type { WorkspacesPageState } from "../use-workspaces-page"

export function createWorkspaceColumns(
  page: WorkspacesPageState,
  t: CommerceTranslate
): CommerceColumn<WorkspaceResponse>[] {
  return [
    {
      key: "title",
      header: t("commerce.workspaces.columns.workspace"),
      render: (item) => (
        <button
          type="button"
          className="block w-full min-w-0 text-left"
          onClick={() => page.openWorkspaceDetail(item.id)}
        >
          <span className="block truncate font-medium">{item.title}</span>
          <IdText>{item.id}</IdText>
        </button>
      ),
    },
    {
      key: "type",
      header: t("commerce.common.type"),
      render: (item) => getWorkspaceTypeLabel(t, item.workspaceType),
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
      width: "240px",
      render: (item) => (
        <div className="flex flex-wrap gap-2">
          <Button
            size="xs"
            variant="outline"
            onClick={() => page.openWorkspaceDetail(item.id)}
          >
            {t("commerce.workspaces.panels.viewDetail")}
          </Button>
          <Button
            size="xs"
            variant="outline"
            onClick={() => page.openWorkspaceEdit(item)}
          >
            {t("commerce.common.edit")}
          </Button>
          <Button
            size="xs"
            variant="destructive"
            onClick={() => {
              const confirmed = window.confirm(
                t("commerce.workspaces.confirm.deleteWorkspace", {
                  title: item.title,
                })
              )
              if (confirmed) {
                page.archiveWorkspace(item.id)
              }
            }}
          >
            {t("commerce.common.delete")}
          </Button>
        </div>
      ),
    },
  ]
}

export function createTaskColumns(
  page: WorkspacesPageState,
  t: CommerceTranslate
): CommerceColumn<WorkspaceTaskResponse>[] {
  return [
    {
      key: "title",
      header: t("commerce.workspaces.columns.task"),
      render: (item) => (
        <div className="w-full min-w-0">
          <div className="truncate font-medium">{item.title}</div>
          <span className="block truncate text-sm text-muted-foreground">
            {item.ownerRole
              ? getCommerceEnumLabel(t, "ownerRole", item.ownerRole)
              : item.description || "-"}
          </span>
        </div>
      ),
    },
    {
      key: "priority",
      header: t("commerce.workspaces.columns.priority"),
      render: (item) => item.priority ?? 0,
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      width: "10rem",
      render: (item) => (
        <NativeSelect
          value={item.status}
          options={workspaceTaskStatusOptions(t)}
          onValueChange={(status) =>
            page.updateTaskStatus({ task: item, status })
          }
        />
      ),
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      width: "160px",
      render: (item) => (
        <Button
          size="xs"
          variant="outline"
          onClick={() => page.archiveTask(item.id)}
        >
          {t("commerce.common.archive")}
        </Button>
      ),
    },
  ]
}

export function createRunColumns(
  page: WorkspacesPageState,
  t: CommerceTranslate
): CommerceColumn<WorkspaceRunResponse>[] {
  return [
    {
      key: "goal",
      header: t("commerce.workspaces.columns.runGoal"),
      render: (item) => (
        <div className="w-full min-w-0">
          <span className="block truncate font-medium">{item.goal}</span>
          <span className="block truncate text-sm text-muted-foreground">
            {getCommerceEnumLabel(t, "source", item.source || "manual")}
          </span>
        </div>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      width: "10rem",
      render: (item) => (
        <NativeSelect
          value={item.status}
          options={workspaceRunStatusOptions(t)}
          onValueChange={(status) =>
            page.updateRunStatus({ run: item, status })
          }
        />
      ),
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      width: "160px",
      render: (item) => (
        <Button
          size="xs"
          variant="outline"
          onClick={() => page.openRunDetailDrawer(item.id)}
        >
          {t("commerce.workspaces.panels.viewDetail")}
        </Button>
      ),
    },
  ]
}

export function createRunTaskColumns(
  page: WorkspacesPageState,
  t: CommerceTranslate
): CommerceColumn<WorkspaceRunTaskResponse>[] {
  return [
    {
      key: "task",
      header: t("commerce.workspaces.columns.snapshot"),
      render: (item) => (
        <div className="w-full min-w-0">
          <div className="truncate font-medium">#{item.seq ?? "-"}</div>
          <span className="block truncate text-sm text-muted-foreground">
            {item.ownerRole
              ? getCommerceEnumLabel(t, "ownerRole", item.ownerRole)
              : item.workspaceTaskId || "-"}
          </span>
        </div>
      ),
    },
    {
      key: "acceptance",
      header: t("commerce.workspaces.columns.acceptance"),
      render: (item) =>
        item.acceptancePassed
          ? t("commerce.common.approve")
          : (item.acceptanceScore ?? "-"),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      width: "10rem",
      render: (item) => (
        <NativeSelect
          value={item.status}
          options={workspaceRunTaskStatusOptions(t)}
          onValueChange={(status) =>
            page.updateRunTaskStatus({ runTask: item, status })
          }
        />
      ),
    },
  ]
}

export { createArtifactColumns } from "./workspace-artifact-columns"
export { createExecutionColumns } from "./workspace-execution-columns"
