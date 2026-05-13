import { useTranslation } from "react-i18next"
import {
  CommercePanel,
  CommerceTable,
  MutedText,
} from "../../components/commerce-ui"
import type { WorkspacesPageState } from "../use-workspaces-page"
import { createSkillRunColumns } from "./workspace-skill-run-columns"

export function WorkspaceSkillRunsPanel({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.workspaces.panels.skillRuns")}
      description={t("commerce.workspaces.panels.skillRunsDesc")}
    >
      {page.selectedRun ? (
        <div className="mb-3 min-w-0 rounded-lg border border-(--app-border) bg-muted/30 p-3">
          <div className="truncate text-sm font-medium">{page.selectedRun.goal}</div>
          <MutedText>
            {page.selectedRun.source || "manual"} / {page.selectedRun.status}
          </MutedText>
        </div>
      ) : null}
      <CommerceTable
        loading={page.loading}
        items={page.skillRuns}
        emptyText={t("commerce.workspaces.empty.skillRuns")}
        getRowId={(item) => item.id}
        columns={createSkillRunColumns(t)}
      />
    </CommercePanel>
  )
}
