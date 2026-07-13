import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import { Link, useParams } from "react-router"
import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../../components/commerce-ui"
import { WorkspaceDialogs } from "../components/workspace-dialogs"
import {
  WorkspaceDetailPanel,
  WorkspaceDetailSummary,
  WorkspaceMetrics,
} from "../components/workspace-panels"
import { useWorkspacesPage } from "../use-workspaces-page"
import { filterWorkspaceDetailMetricCards } from "../workspace-metrics"

export default function WorkspaceDetailPage() {
  const { t } = useTranslation()
  const { workspaceId } = useParams<{ workspaceId: string }>()
  const page = useWorkspacesPage({ workspaceId: workspaceId ?? null })
  const metricCards = useMemo(
    () => filterWorkspaceDetailMetricCards(page.metricCards),
    [page.metricCards]
  )

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("commerce.pages.workspaces.eyebrow")}
        title={t("commerce.pages.workspaces.title")}
        description={t("commerce.pages.workspaces.description")}
      />
      <CommerceBody>
        <WorkspaceDetailBreadcrumb
          listLabel={t("commerce.pages.workspaces.title")}
          currentLabel={
            page.selectedWorkspace?.title ?? t("commerce.workspaces.panels.detail")
          }
        />
        <WorkspaceDetailSummary page={page} />
        <WorkspaceMetrics items={metricCards} />
        <WorkspaceDetailPanel page={page} />
      </CommerceBody>
      <WorkspaceDialogs page={page} />
    </CommerceSurface>
  )
}

function WorkspaceDetailBreadcrumb({
  currentLabel,
  listLabel,
}: {
  currentLabel: string
  listLabel: string
}) {
  return (
    <nav aria-label="breadcrumb">
      <ol className="flex flex-wrap items-center gap-1.5 text-sm text-muted-foreground wrap-break-word">
        <li className="inline-flex items-center gap-1">
          <Link className="transition-colors hover:text-foreground" to="/workspaces">
            {listLabel}
          </Link>
        </li>
        <li aria-hidden="true" className="inline-flex items-center gap-1">
          &gt;
        </li>
        <li className="inline-flex min-w-0 items-center gap-1">
          <span
            aria-current="page"
            className="truncate font-normal text-foreground"
            role="link"
          >
            {currentLabel}
          </span>
        </li>
      </ol>
    </nav>
  )
}
