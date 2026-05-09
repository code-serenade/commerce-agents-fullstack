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
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
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
          className="block min-w-0 text-left"
          onClick={() => page.setSelectedWorkspaceId(item.id)}
        >
          <span className="block truncate font-medium">{item.title}</span>
          <IdText>{item.id}</IdText>
        </button>
      ),
    },
    {
      key: "type",
      header: t("commerce.common.type"),
      render: (item) => item.workspaceType || "general",
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>{item.status}</StatusBadge>
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
        <div className="min-w-0">
          <div className="truncate font-medium">{item.title}</div>
          <MutedText>{item.ownerRole || item.description || "-"}</MutedText>
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
      render: (item) => (
        <NativeSelect
          value={item.status}
          options={taskStatusOptions}
          onValueChange={(status) => page.updateTaskStatus({ task: item, status })}
        />
      ),
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      render: (item) => (
        <Button size="xs" variant="outline" onClick={() => page.archiveTask(item.id)}>
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
        <button
          type="button"
          className="block min-w-0 text-left"
          onClick={() => page.setSelectedRunId(item.id)}
        >
          <span className="block truncate font-medium">{item.goal}</span>
          <MutedText>{item.source || "manual"}</MutedText>
        </button>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <NativeSelect
          value={item.status}
          options={runStatusOptions}
          onValueChange={(status) => page.updateRunStatus({ run: item, status })}
        />
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
        <div className="min-w-0">
          <div className="truncate font-medium">#{item.seq ?? "-"}</div>
          <MutedText>{item.ownerRole || item.workspaceTaskId || "-"}</MutedText>
        </div>
      ),
    },
    {
      key: "acceptance",
      header: t("commerce.workspaces.columns.acceptance"),
      render: (item) => (item.acceptancePassed ? t("commerce.common.approve") : item.acceptanceScore ?? "-"),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <NativeSelect
          value={item.status}
          options={runTaskStatusOptions}
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
      render: (item) => (
        <button
          type="button"
          className="block min-w-0 text-left"
          onClick={() => page.setSelectedArtifactId(item.id)}
        >
          <span className="block truncate font-medium">{item.title}</span>
          <MutedText>{item.preview || item.fileName || item.id}</MutedText>
        </button>
      ),
    },
    {
      key: "type",
      header: t("commerce.common.type"),
      render: (item) => item.artifactType,
    },
    {
      key: "validation",
      header: t("commerce.workspaces.columns.validation"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.validationStatus)}>
          {item.validationStatus}
        </StatusBadge>
      ),
    },
  ]
}

export { createExecutionColumns } from "./workspace-execution-columns"

const taskStatusOptions = [
  { label: String("pending"), value: "pending" },
  { label: String("in_progress"), value: "in_progress" },
  { label: String("done"), value: "done" },
  { label: String("blocked"), value: "blocked" },
  { label: String("cancelled"), value: "cancelled" },
]

const runStatusOptions = [
  { label: String("planned"), value: "planned" },
  { label: String("running"), value: "running" },
  { label: String("succeeded"), value: "succeeded" },
  { label: String("failed"), value: "failed" },
  { label: String("cancelled"), value: "cancelled" },
]

const runTaskStatusOptions = [
  { label: String("pending"), value: "pending" },
  { label: String("running"), value: "running" },
  { label: String("succeeded"), value: "succeeded" },
  { label: String("failed"), value: "failed" },
  { label: String("skipped"), value: "skipped" },
  { label: String("cancelled"), value: "cancelled" },
]
