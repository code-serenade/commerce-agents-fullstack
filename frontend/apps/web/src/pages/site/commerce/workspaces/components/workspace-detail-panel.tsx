import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  Button,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@workspace/ui-components"
import {
  CommerceDetailDrawer,
  CommercePanel,
  CommerceTable,
  MutedText,
  SubPanelAction,
} from "../../components/commerce-ui"
import {
  getCommerceEnumLabel,
  getCommerceStatusLabel,
} from "../../domain/enum-mappings"
import type { WorkspacesPageState } from "../use-workspaces-page"
import {
  createExecutionColumns,
  createRunColumns,
  createRunTaskColumns,
  createTaskColumns,
} from "./workspace-table-columns"
import { WorkspaceArtifactsTab } from "./workspace-artifacts-tab"
import { WorkspaceQualityContent } from "./workspace-quality-panel"
import { WorkspaceSkillRunsContent } from "./workspace-skill-runs-panel"

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
    <CommercePanel>
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
        buttonSize="default"
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
      <CommerceDetailDrawer
        open={page.detailDrawer === "run"}
        closeLabel={t("commerce.common.close")}
        title={
          page.selectedRun?.goal ?? t("commerce.workspaces.panels.runSnapshot")
        }
        description={t("commerce.workspaces.panels.runSnapshotDesc")}
        onClose={page.closeDetailDrawer}
      >
        <RunSnapshotDetail page={page} />
      </CommerceDetailDrawer>
    </TabsContent>
  )
}

function RunSnapshotDetail({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = useState("snapshot")

  return (
    <Tabs value={activeTab} onValueChange={setActiveTab}>
      <TabsList>
        <TabsTrigger value="snapshot">
          {t("commerce.workspaces.panels.runSnapshot")}
        </TabsTrigger>
        <TabsTrigger value="skill-runs">
          {t("commerce.workspaces.panels.skillRuns")}
        </TabsTrigger>
        <TabsTrigger value="quality">
          {t("commerce.workspaces.panels.qualityLoop")}
        </TabsTrigger>
      </TabsList>
      <TabsContent value="snapshot">
        <div className="space-y-3">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="min-w-0">
              <h3 className="truncate text-sm font-semibold">
                {t("commerce.workspaces.panels.runSnapshot")}
              </h3>
              <MutedText>
                {t("commerce.workspaces.panels.runSnapshotDesc")}
              </MutedText>
            </div>
            <Button
              size="default"
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
      </TabsContent>
      <TabsContent value="skill-runs">
        <div className="space-y-3">
          <div>
            <h3 className="text-sm font-semibold">
              {t("commerce.workspaces.panels.skillRuns")}
            </h3>
            <MutedText>{t("commerce.workspaces.panels.skillRunsDesc")}</MutedText>
          </div>
          <WorkspaceSkillRunsContent page={page} />
        </div>
      </TabsContent>
      <TabsContent value="quality">
        <div className="space-y-3">
          <div>
            <h3 className="text-sm font-semibold">
              {t("commerce.workspaces.panels.qualityLoop")}
            </h3>
            <MutedText>
              {t("commerce.workspaces.panels.qualityLoopDesc")}
            </MutedText>
          </div>
          <WorkspaceQualityContent page={page} />
        </div>
      </TabsContent>
    </Tabs>
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
                {getCommerceEnumLabel(
                  t,
                  "actionType",
                  page.focusedExecution.actionType
                )}
              </h3>
              <MutedText>{page.focusedExecution.description}</MutedText>
            </div>
            <MutedText>
              {getCommerceStatusLabel(t, page.focusedExecution.status)}
            </MutedText>
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
