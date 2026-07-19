import { useMemo, type ReactNode } from "react"
import { useTranslation } from "react-i18next"
import { Link, useParams } from "react-router"
import { Archive } from "lucide-react"
import { Button } from "@workspace/ui-components"
import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../../components/commerce-ui"
import { WorkspaceDialogs } from "../components/workspace-dialogs"
import {
  WorkspaceDetailPanel,
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
          ariaLabel={t("commerce.common.breadcrumb")}
          listLabel={t("commerce.pages.workspaces.title")}
          currentLabel={
            page.selectedWorkspace?.title ??
            t("commerce.workspaces.panels.detail")
          }
          actions={
            <Button
              variant="destructive"
              disabled={!page.selectedWorkspace}
              onClick={() => {
                if (page.selectedWorkspace) {
                  void page.archiveWorkspace(page.selectedWorkspace.id)
                }
              }}
            >
              <Archive className="size-4" />
              {t("commerce.workspaces.panels.archiveWorkspace")}
            </Button>
          }
        />
        <WorkspaceMetrics items={metricCards} />
        <WorkspaceDetailPanel page={page} />
      </CommerceBody>
      <WorkspaceDialogs page={page} />
    </CommerceSurface>
  )
}

function WorkspaceDetailBreadcrumb({
  ariaLabel,
  actions,
  currentLabel,
  listLabel,
}: {
  ariaLabel: string
  actions?: ReactNode
  currentLabel: string
  listLabel: string
}) {
  return (
    <div className="flex min-w-0 flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
      <nav aria-label={ariaLabel} className="min-w-0">
        <ol className="flex flex-wrap items-center gap-1.5 text-sm text-muted-foreground wrap-break-word">
          <li className="inline-flex items-center gap-1">
            <Link
              className="transition-colors hover:text-foreground"
              to="/workspaces"
            >
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
      {actions ? <div className="shrink-0">{actions}</div> : null}
    </div>
  )
}
