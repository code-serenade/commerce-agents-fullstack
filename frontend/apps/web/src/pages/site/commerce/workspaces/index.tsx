import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../components/commerce-ui"
import { ClipboardList } from "lucide-react"
import { useTranslation } from "react-i18next"
import { DocumentIntroCard } from "../components/document-section"
import { WorkspaceDialogs } from "./components/workspace-dialogs"
import {
  WorkspaceDetailPanel,
  WorkspaceMetrics,
  WorkspaceQualityPanel,
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
        <DocumentIntroCard
          title={t("commerce.pages.workspaces.boundaryTitle")}
          description={t("commerce.pages.workspaces.boundaryDescription")}
          icon={<ClipboardList className="size-4" />}
        />
        <WorkspaceMetrics items={page.metricCards} />
        <div className="grid gap-4 xl:grid-cols-[minmax(22rem,0.85fr)_minmax(0,1.65fr)]">
          <WorkspacesMasterPanel page={page} />
          <WorkspaceDetailPanel page={page} />
        </div>
        <WorkspaceSkillRunsPanel page={page} />
        <WorkspaceQualityPanel page={page} />
      </CommerceBody>
      <WorkspaceDialogs page={page} />
    </CommerceSurface>
  )
}
