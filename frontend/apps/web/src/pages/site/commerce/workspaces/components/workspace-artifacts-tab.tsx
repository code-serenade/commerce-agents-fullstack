import { useTranslation } from "react-i18next"
import { TabsContent } from "@workspace/ui-components"
import {
  CommerceTable,
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { WorkspacesPageState } from "../use-workspaces-page"
import { createArtifactColumns } from "./workspace-table-columns"

export function WorkspaceArtifactsTab({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()
  const artifact = page.selectedArtifact
  const content = artifact?.content || artifact?.preview

  return (
    <TabsContent value="artifacts">
      <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(22rem,0.85fr)]">
        <div className="min-w-0">
          <CommerceTable
            loading={page.loading}
            items={page.artifacts}
            emptyText={t("commerce.workspaces.empty.artifacts")}
            getRowId={(item) => item.id}
            columns={createArtifactColumns(page, t)}
          />
        </div>
        <div className="min-w-0 rounded-lg border border-(--app-border) p-3">
          {artifact ? (
            <div className="space-y-3">
              <div className="flex flex-wrap items-start justify-between gap-2">
                <div className="min-w-0">
                  <h3 className="truncate text-sm font-semibold">
                    {artifact.title}
                  </h3>
                  <MutedText>
                    {artifact.artifactType} / {artifact.source}
                  </MutedText>
                </div>
                <StatusBadge tone={toneForStatus(artifact.validationStatus)}>
                  {artifact.validationStatus}
                </StatusBadge>
              </div>
              <ArtifactMetaGrid page={page} />
              <div className="max-h-112 overflow-auto rounded-lg border border-(--app-border) bg-muted/30 p-3">
                <pre className="whitespace-pre-wrap wrap-break-word text-sm/6 text-foreground">
                  {content || t("commerce.workspaces.empty.artifactContent")}
                </pre>
              </div>
            </div>
          ) : (
            <div className="flex min-h-40 items-center justify-center rounded-lg border border-dashed border-(--app-border) px-4 text-center text-sm text-muted-foreground">
              {t("commerce.workspaces.empty.artifactDetail")}
            </div>
          )}
        </div>
      </div>
    </TabsContent>
  )
}

function ArtifactMetaGrid({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()
  const artifact = page.selectedArtifact

  if (!artifact) return null

  return (
    <div className="grid gap-2 text-xs text-muted-foreground sm:grid-cols-2">
      <ArtifactMeta
        label={t("commerce.workspaces.panels.artifactFormat")}
        value={artifact.format}
      />
      <ArtifactMeta
        label={t("commerce.workspaces.panels.artifactStorage")}
        value={artifact.storageKind}
      />
      <ArtifactMeta
        label={t("commerce.workspaces.panels.artifactFile")}
        value={artifact.fileName || artifact.mimeType || "-"}
      />
      <ArtifactMeta
        label={t("commerce.workspaces.panels.artifactRun")}
        value={artifact.runId || "-"}
      />
    </div>
  )
}

function ArtifactMeta({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0 rounded-md bg-muted/30 px-2 py-1.5">
      <div className="text-[11px] text-muted-foreground">{label}</div>
      <div className="truncate font-mono text-xs text-foreground">{value}</div>
    </div>
  )
}
