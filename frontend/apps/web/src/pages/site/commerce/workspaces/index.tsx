import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../components/commerce-ui"
import { Button } from "@workspace/ui-components"
import { ClipboardList, Plus } from "lucide-react"
import { useTranslation } from "react-i18next"
import { DocumentIntroCard } from "../components/document-section"
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
      />
      <CommerceBody>
        <DocumentIntroCard
          title={t("commerce.workspaces.panels.master")}
          description={t("commerce.workspaces.panels.masterDesc")}
          icon={<ClipboardList className="size-4" />}
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
        <WorkspacesMasterPanel page={page} />
      </CommerceBody>
      <WorkspaceDialogs page={page} />
    </CommerceSurface>
  )
}
