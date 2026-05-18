import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useWorkspaceActions } from "./use-workspace-actions"
import { buildWorkspaceMetricCards } from "./workspace-metrics"
import { useWorkspaceQueries } from "./use-workspace-queries"

export function useWorkspacesPage() {
  const { t } = useTranslation()
  const [keyword, setKeyword] = useState("")
  const [selectedWorkspaceId, setSelectedWorkspaceId] = useState<string | null>(null)
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null)
  const [selectedArtifactId, setSelectedArtifactId] = useState<string | null>(null)
  const [dialog, setDialog] = useState<WorkspaceDialogName | null>(null)

  const queries = useWorkspaceQueries({
    keyword,
    selectedArtifactId,
    selectedRunId,
    selectedWorkspaceId,
  })
  const actions = useWorkspaceActions({
    selectedRunId: queries.selectedRun?.id,
    setDialog,
    setSelectedRunId,
    setSelectedWorkspaceId,
    t,
    workspaceId: queries.selectedWorkspace?.id,
  })
  const metricCards = useMemo(
    () =>
      buildWorkspaceMetricCards(t, {
        executions: queries.executionQueue,
        artifacts: queries.artifacts,
        runs: queries.runs,
        tasks: queries.tasks,
        workspaces: queries.workspaces,
      }),
    [
      queries.artifacts,
      queries.executionQueue,
      queries.runs,
      queries.tasks,
      queries.workspaces,
      t,
    ]
  )

  return {
    ...actions,
    ...queries,
    dialog,
    keyword,
    metricCards,
    setSelectedArtifactId,
    setDialog,
    setKeyword,
    setSelectedRunId,
    setSelectedWorkspaceId,
  }
}

export type WorkspaceDialogName =
  | "create-workspace"
  | "create-task"
  | "create-run"
  | "create-run-task"
  | "create-execution"

export type WorkspacesPageState = ReturnType<typeof useWorkspacesPage>
