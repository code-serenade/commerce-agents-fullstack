import { useTranslation } from "react-i18next"
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
    <CommercePanel>
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
