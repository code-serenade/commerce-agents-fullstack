import { useTranslation } from "react-i18next"
import {
  CommercePanel,
  CommerceTable,
  MutedText,
} from "../../components/commerce-ui"
import {
  getCommerceEnumLabel,
  getCommerceStatusLabel,
} from "../../domain/enum-mappings"
import type { WorkspacesPageState } from "../use-workspaces-page"
import { createSkillRunColumns } from "./workspace-skill-run-columns"

export function WorkspaceSkillRunsPanel({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.workspaces.panels.skillRuns")}
      description={t("commerce.workspaces.panels.skillRunsDesc")}
    >
      <WorkspaceSkillRunsContent page={page} />
    </CommercePanel>
  )
}

export function WorkspaceSkillRunsContent({
  page,
}: {
  page: WorkspacesPageState
}) {
  const { t } = useTranslation()

  return (
    <div className="space-y-3">
      {page.selectedRun ? (
        <div className="min-w-0 rounded-lg border border-(--app-border) bg-muted/30 p-3">
          <div className="truncate text-sm font-medium">
            {page.selectedRun.goal}
          </div>
          <MutedText>
            {getCommerceEnumLabel(
              t,
              "source",
              page.selectedRun.source || "manual"
            )}{" "}
            / {getCommerceStatusLabel(t, page.selectedRun.status)}
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
    </div>
  )
}
