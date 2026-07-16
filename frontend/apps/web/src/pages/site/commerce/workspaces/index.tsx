import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../components/commerce-ui"
import { Button } from "@workspace/ui-components"
import { Plus } from "lucide-react"
import { useTranslation } from "react-i18next"
import { WorkspaceDialogs } from "./components/workspace-dialogs"
import { WorkspacesMasterPanel } from "./components/workspace-panels"
import { useWorkspacesPage } from "./use-workspaces-page"

export default function WorkspacesPage() {
  const { t } = useTranslation()
  const page = useWorkspacesPage()

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("commerce.pages.workspaces.eyebrow")}
        title={t("commerce.pages.workspaces.title")}
        description={t("commerce.pages.workspaces.description")}
        actions={
          <Button
            size="lg"
            variant="primary"
            onClick={() => page.setDialog("create-workspace")}
          >
            <Plus className="size-4" />
            {t("commerce.workspaces.panels.createWorkspace")}
          </Button>
        }
      />
      <CommerceBody>
        <WorkspacesMasterPanel page={page} />
      </CommerceBody>
      <WorkspaceDialogs page={page} />
    </CommerceSurface>
  )
}
