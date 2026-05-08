import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import {
  CommercePanel,
  CommerceTable,
  CommerceToolbar,
} from "../../components/commerce-ui"
import type { WorkspacesPageState } from "../use-workspaces-page"
import { createWorkspaceColumns } from "./workspace-table-columns"

export function WorkspacesMasterPanel({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.workspaces.panels.master")}
      description={t("commerce.workspaces.panels.masterDesc")}
      actions={
        <Button size="sm" onClick={() => page.setDialog("create-workspace")}>
          {t("commerce.workspaces.panels.createWorkspace")}
        </Button>
      }
    >
      <div className="space-y-3">
        <CommerceToolbar
          keyword={page.keyword}
          keywordPlaceholder={t("commerce.workspaces.panels.search")}
          onKeywordChange={page.setKeyword}
        />
        <CommerceTable
          loading={page.loading}
          items={page.workspaces}
          emptyText={t("commerce.workspaces.panels.empty")}
          getRowId={(item) => item.id}
          columns={createWorkspaceColumns(page, t)}
        />
      </div>
    </CommercePanel>
  )
}
