import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
  InlineAlert,
} from "../components/commerce-ui"
import { useTranslation } from "react-i18next"
import { WorkspaceDialogs } from "./components/workspace-dialogs"
import {
  WorkspaceDetailPanel,
  WorkspaceMetrics,
  WorkspaceSkillRunsPanel,
  WorkspacesMasterPanel,
} from "./components/workspace-panels"
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
        <InlineAlert
          variant="default"
          title={t("commerce.pages.workspaces.boundaryTitle")}
          description={t("commerce.pages.workspaces.boundaryDescription")}
        />
        <WorkspaceMetrics items={page.metricCards} />
        <div className="grid gap-4 xl:grid-cols-[minmax(22rem,0.85fr)_minmax(0,1.65fr)]">
          <WorkspacesMasterPanel page={page} />
          <WorkspaceDetailPanel page={page} />
        </div>
        <WorkspaceSkillRunsPanel page={page} />
      </CommerceBody>
      <WorkspaceDialogs page={page} />
    </CommerceSurface>
  )
}
