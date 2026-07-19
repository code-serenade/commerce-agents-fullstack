import { Button } from "@workspace/ui-components"
import type { WorkspaceArtifactResponse } from "@/api"
import {
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { WorkspacesPageState } from "../use-workspaces-page"
import { getWorkspaceArtifactDisplay } from "../view-model/workspace-artifact-display-view-model"

export function createArtifactColumns(
  page: WorkspacesPageState,
  t: CommerceTranslate
): CommerceColumn<WorkspaceArtifactResponse>[] {
  return [
    {
      key: "artifact",
      header: t("commerce.workspaces.columns.artifact"),
      render: (item) => {
        const display = getWorkspaceArtifactDisplay(t, item)
        return (
          <div className="w-full min-w-0">
            <span className="block truncate font-medium">{display.title}</span>
            <span className="mt-1 line-clamp-2 block overflow-hidden text-sm/5 text-muted-foreground">
              {item.preview || display.source}
            </span>
          </div>
        )
      },
    },
    {
      key: "type",
      header: t("commerce.common.type"),
      width: "10rem",
      render: (item) => getWorkspaceArtifactDisplay(t, item).type,
    },
    {
      key: "validation",
      header: t("commerce.workspaces.columns.validation"),
      width: "8rem",
      render: (item) => {
        const display = getWorkspaceArtifactDisplay(t, item)
        return (
          <StatusBadge tone={toneForStatus(item.validationStatus)}>
            {display.validationStatus}
          </StatusBadge>
        )
      },
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      width: "160px",
      render: (item) => (
        <Button
          size="xs"
          variant="outline"
          onClick={() => page.openArtifactDetailDrawer(item.id)}
        >
          {t("commerce.workspaces.panels.viewDetail")}
        </Button>
      ),
    },
  ]
}
