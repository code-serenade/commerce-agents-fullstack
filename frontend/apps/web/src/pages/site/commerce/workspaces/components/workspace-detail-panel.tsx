import { useState } from "react"
import { useTranslation } from "react-i18next"
import { Button, Tabs, TabsContent, TabsList, TabsTrigger } from "@workspace/ui-components"
import {
  CommercePanel,
  CommerceTable,
  MutedText,
  SubPanelAction,
} from "../../components/commerce-ui"
import type { WorkspacesPageState } from "../use-workspaces-page"
import {
  createExecutionColumns,
  createRunColumns,
  createRunTaskColumns,
  createTaskColumns,
} from "./workspace-table-columns"
import { WorkspaceArtifactsTab } from "./workspace-artifacts-tab"

export function WorkspaceDetailPanel({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = useState("tasks")

  if (!page.selectedWorkspace) {
    return (
      <CommercePanel
        title={t("commerce.workspaces.panels.detail")}
        description={t("commerce.workspaces.panels.detailDesc")}
      >
        <div className="rounded-lg border border-dashed border-(--app-border) p-8 text-center text-sm text-muted-foreground">
          {t("commerce.workspaces.panels.noWorkspace")}
        </div>
      </CommercePanel>
    )
  }

  return (
    <CommercePanel
      title={page.selectedWorkspace.title}
      description={t("commerce.workspaces.panels.selectedDesc")}
      actions={
        <Button
          size="sm"
          variant="destructive"
          onClick={() => page.archiveWorkspace(page.selectedWorkspace!.id)}
        >
          {t("commerce.workspaces.panels.archiveWorkspace")}
        </Button>
      }
    >
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="tasks">
            {t("commerce.workspaces.panels.tasksTab")}
          </TabsTrigger>
          <TabsTrigger value="runs">
            {t("commerce.workspaces.panels.runsTab")}
          </TabsTrigger>
          <TabsTrigger value="artifacts">
            {t("commerce.workspaces.panels.artifactsTab")}
          </TabsTrigger>
          <TabsTrigger value="executions">
            {t("commerce.workspaces.panels.executionsTab")}
          </TabsTrigger>
        </TabsList>
        <WorkspaceTasksTab page={page} />
        <WorkspaceRunsTab page={page} />
        <WorkspaceArtifactsTab page={page} />
        <WorkspaceExecutionsTab page={page} />
      </Tabs>
    </CommercePanel>
  )
}

function WorkspaceTasksTab({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()

  return (
    <TabsContent value="tasks">
      <SubPanelAction
        label={t("commerce.workspaces.panels.createTask")}
        onClick={() => page.setDialog("create-task")}
      />
      <CommerceTable
        loading={page.loading}
        items={page.tasks}
        emptyText={t("commerce.workspaces.empty.tasks")}
        getRowId={(item) => item.id}
        columns={createTaskColumns(page, t)}
      />
    </TabsContent>
  )
}

function WorkspaceRunsTab({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()

  return (
    <TabsContent value="runs">
      <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(22rem,0.8fr)]">
        <div className="min-w-0">
          <SubPanelAction
            label={t("commerce.workspaces.panels.createRun")}
            onClick={() => page.setDialog("create-run")}
          />
          <CommerceTable
            loading={page.loading}
            items={page.runs}
            emptyText={t("commerce.workspaces.empty.runs")}
            getRowId={(item) => item.id}
            columns={createRunColumns(page, t)}
          />
        </div>
        <div className="min-w-0 rounded-lg border border-(--app-border) p-3">
          <div className="mb-3 flex items-center justify-between gap-2">
            <div className="min-w-0">
              <h3 className="truncate text-sm font-semibold">
                {page.selectedRun?.goal ??
                  t("commerce.workspaces.panels.runSnapshot")}
              </h3>
              <MutedText>
                {t("commerce.workspaces.panels.runSnapshotDesc")}
              </MutedText>
            </div>
            <Button
              size="sm"
              variant="outline"
              disabled={!page.selectedRun}
              onClick={() => page.setDialog("create-run-task")}
            >
              {t("commerce.workspaces.panels.createSnapshot")}
            </Button>
          </div>
          <CommerceTable
            loading={page.loading}
            items={page.runTasks}
            emptyText={t("commerce.workspaces.empty.runTasks")}
            getRowId={(item) => item.id}
            columns={createRunTaskColumns(page, t)}
          />
        </div>
      </div>
    </TabsContent>
  )
}

function WorkspaceExecutionsTab({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()

  return (
    <TabsContent value="executions">
      {page.focusedExecution ? (
        <div className="mb-3 rounded-lg border border-(--app-border) bg-muted/30 p-3">
          <div className="flex flex-wrap items-start justify-between gap-2">
            <div className="min-w-0">
              <h3 className="truncate text-sm font-semibold">
                {page.focusedExecution.actionType}
              </h3>
              <MutedText>{page.focusedExecution.description}</MutedText>
            </div>
            <MutedText>{page.focusedExecution.status}</MutedText>
          </div>
          {page.focusedExecution.decisionReason ? (
            <div className="mt-2 text-sm text-muted-foreground">
              {page.focusedExecution.decisionReason}
            </div>
          ) : null}
        </div>
      ) : null}
      <SubPanelAction
        label={t("commerce.workspaces.panels.createExecution")}
        onClick={() => page.setDialog("create-execution")}
      />
      <CommerceTable
        loading={page.loading}
        items={page.executionQueue}
        emptyText={t("commerce.workspaces.empty.executions")}
        getRowId={(item) => item.id}
        columns={createExecutionColumns(page, t)}
      />
    </TabsContent>
  )
}
