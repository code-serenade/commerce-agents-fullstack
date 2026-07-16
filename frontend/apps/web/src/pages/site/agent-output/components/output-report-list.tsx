import { ChevronRight, FileText, RefreshCcw } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import { MutedText } from "@/pages/site/commerce/components/commerce-ui"
import {
  formatOutputBackendNotice,
  getOutputArtifactDisplay,
} from "../view-model/output-artifact-display-view-model"
import type { AgentOutputBackendState } from "../use-agent-output-page"
import { EmptyState, LoadingState } from "./output-backend-deliverable-blocks"
import { OutputValidationBadge } from "./output-validation-badge"

export function OutputReportList({
  backend,
  mobileHidden,
  onRefresh,
  onSelectArtifact,
}: {
  backend: AgentOutputBackendState
  mobileHidden: boolean
  onRefresh: () => void
  onSelectArtifact: (artifactId: string) => void
}) {
  const { t } = useTranslation()
  const selectedId =
    backend.selectedArtifact?.id ?? backend.selectedArtifactId ?? ""

  return (
    <aside
      className={`min-h-0 min-w-0 flex-col overflow-hidden border-b border-(--app-border) bg-white md:flex md:border-r md:border-b-0 md:border-border/50 dark:bg-background ${
        mobileHidden ? "hidden" : "flex"
      }`}
    >
      <div className="flex min-w-0 shrink-0 items-center justify-between gap-3 border-b border-(--app-border) px-4 py-3 md:border-border/60 md:bg-muted/10 md:px-3">
        <div className="min-w-0">
          <h2 className="truncate text-base font-semibold md:text-sm md:font-medium md:text-muted-foreground">
            {t("agentOutput.backend.listTitle")}
          </h2>
          <MutedText>
            {t("agentOutput.backend.listDesc", {
              artifacts: backend.deliverables.length,
              workspaces: backend.workspaces.length,
            })}
          </MutedText>
        </div>
        <Button
          type="button"
          variant="link"
          size="sm"
          className="text-primary hover:text-primary active:text-primary"
          onClick={onRefresh}
        >
          <RefreshCcw className="size-4" />
          {t("agentOutput.backend.refresh")}
        </Button>
      </div>

      {backend.loading ? (
        <LoadingState label={t("agentOutput.backend.loading")} />
      ) : backend.errorMessage ? (
        <div className="p-4">
          <EmptyState
            title={t("agentOutput.backend.errorTitle")}
            detail={formatOutputBackendNotice(t, backend.errorMessage)}
          />
        </div>
      ) : backend.deliverables.length ? (
        <div className="min-h-0 flex-1 overflow-y-auto [scrollbar-gutter:stable]">
          {backend.deliverables.map((artifact) => {
            const active = artifact.id === selectedId
            const display = getOutputArtifactDisplay(t, artifact)

            return (
              <button
                key={artifact.id}
                type="button"
                className={`group block min-h-20 w-full min-w-0 cursor-pointer border-b border-(--app-border) px-4 py-3.5 text-left transition-colors md:min-h-15 md:border-border/40 md:px-3 md:py-2.5 ${
                  active
                    ? "bg-white md:bg-muted/35 md:shadow-[inset_2px_0_0_var(--primary)] dark:bg-background"
                    : "bg-white hover:bg-(--shell-sidebar-item-hover-background) dark:bg-background"
                }`}
                onClick={() => onSelectArtifact(artifact.id)}
              >
                <span className="flex min-w-0 items-center gap-3">
                  <span
                    className={`mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground group-hover:text-foreground md:hidden ${
                      active
                        ? "md:text-muted-foreground"
                        : ""
                    }`}
                  >
                    <FileText className="size-4" />
                  </span>
                  <span className="min-w-0 flex-1">
                    <span
                      className={`line-clamp-2 text-sm/5 font-semibold text-foreground md:block md:truncate md:font-medium ${
                        active
                          ? "md:text-foreground"
                          : "md:text-muted-foreground group-hover:text-foreground"
                      }`}
                    >
                      {display.title}
                    </span>
                    <span className="mt-1 flex min-w-0 items-center justify-between gap-2">
                      <span className="truncate text-xs text-muted-foreground md:text-muted-foreground/70">
                        {artifact.workspace.title}
                      </span>
                      <OutputValidationBadge
                        label={display.validationStatus}
                        quietOnDesktop
                        status={artifact.validationStatus}
                      />
                    </span>
                  </span>
                  <ChevronRight className="size-4 shrink-0 text-muted-foreground md:hidden" />
                </span>
              </button>
            )
          })}
        </div>
      ) : (
        <div className="p-4">
          <EmptyState
            title={t("agentOutput.backend.emptyTitle")}
            detail={t("agentOutput.backend.emptyDescription")}
          />
        </div>
      )}
    </aside>
  )
}
