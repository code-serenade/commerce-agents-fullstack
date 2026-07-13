import { Button } from "@workspace/ui-components"
import { Archive, ClipboardList } from "lucide-react"
import { useTranslation } from "react-i18next"
import { DocumentIntroCard } from "../../components/document-section"
import type { WorkspacesPageState } from "../use-workspaces-page"

export function WorkspaceDetailSummary({
  page,
}: {
  page: WorkspacesPageState
}) {
  const { t } = useTranslation()
  const selectedWorkspace = page.selectedWorkspace

  return (
    <DocumentIntroCard
      title={selectedWorkspace?.title ?? t("commerce.workspaces.panels.detail")}
      description={t("commerce.workspaces.panels.selectedDesc")}
      icon={<ClipboardList className="size-4" />}
      actions={
        <Button
          variant="destructive"
          disabled={!selectedWorkspace}
          onClick={() => {
            if (selectedWorkspace) {
              void page.archiveWorkspace(selectedWorkspace.id)
            }
          }}
        >
          <Archive className="size-4" />
          {t("commerce.workspaces.panels.archiveWorkspace")}
        </Button>
      }
    />
  )
}
