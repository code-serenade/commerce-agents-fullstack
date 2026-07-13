import { Button, NativeSelect } from "@workspace/ui-components"
import type {
  WorkspaceArtifactResponse,
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
  getCommerceStatusLabel,
  workspaceRunStatusOptions,
  workspaceRunTaskStatusOptions,
  workspaceTaskStatusOptions,
} from "../../domain/status-options"
import { getWorkspaceTypeLabel } from "../../domain/workspace-types"
import type { WorkspacesPageState } from "../use-workspaces-page"
import { getWorkspaceArtifactDisplay } from "../view-model/workspace-artifact-display-view-model"

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
            {item.ownerRole || item.description || "-"}
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
            {item.source || "manual"}
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
            {item.ownerRole || item.workspaceTaskId || "-"}
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

export function createArtifactColumns(
  page: WorkspacesPageState,
  t: CommerceTranslate
): CommerceColumn<WorkspaceArtifactResponse>[] {
  return [
    {
      key: "artifact",
      header: t("commerce.workspaces.columns.artifact"),
      render: (item) => {
        const display = getWorkspaceArtifactDisplay(t, item)
        return (
          <div className="w-full min-w-0">
            <span className="block truncate font-medium">{display.title}</span>
            <span className="mt-1 line-clamp-2 block overflow-hidden text-sm/5 text-muted-foreground">
              {item.preview || display.source}
            </span>
          </div>
        )
      },
    },
    {
      key: "type",
      header: t("commerce.common.type"),
      width: "10rem",
      render: (item) => getWorkspaceArtifactDisplay(t, item).type,
    },
    {
      key: "validation",
      header: t("commerce.workspaces.columns.validation"),
      width: "8rem",
      render: (item) => {
        const display = getWorkspaceArtifactDisplay(t, item)
        return (
          <StatusBadge tone={toneForStatus(item.validationStatus)}>
            {display.validationStatus}
          </StatusBadge>
        )
      },
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      width: "160px",
      render: (item) => (
        <Button
          size="xs"
          variant="outline"
          onClick={() => page.openArtifactDetailDrawer(item.id)}
        >
          {t("commerce.workspaces.panels.viewDetail")}
        </Button>
      ),
    },
  ]
}

export { createExecutionColumns } from "./workspace-execution-columns"
