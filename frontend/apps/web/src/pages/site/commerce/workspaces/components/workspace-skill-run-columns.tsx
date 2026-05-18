import type { AgentSkillRunResponse } from "@/api"
import {
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"

export function createSkillRunColumns(
  t: CommerceTranslate
): CommerceColumn<AgentSkillRunResponse>[] {
  return [
    {
      key: "skill",
      header: t("commerce.workspaces.columns.skill"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.skillId}</div>
          <MutedText>{item.workerId || item.role || item.requestId}</MutedText>
        </div>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>
          {item.status}
        </StatusBadge>
      ),
    },
    {
      key: "mode",
      header: t("commerce.workspaces.columns.executionMode"),
      render: (item) => (
        <div className="space-y-1">
          <MutedText>{item.executionMode || "-"}</MutedText>
          <MutedText>{item.riskLevel || "-"}</MutedText>
        </div>
      ),
    },
    {
      key: "summary",
      header: t("commerce.workspaces.columns.summary"),
      render: (item) => (
        <div className="max-w-80 space-y-1">
          <div className="truncate text-sm">
            {item.summary || item.reason || item.error || "-"}
          </div>
          <MutedText>
            {item.completedAt || item.requestedAt || item.createdAt || "-"}
          </MutedText>
        </div>
      ),
    },
  ]
}
