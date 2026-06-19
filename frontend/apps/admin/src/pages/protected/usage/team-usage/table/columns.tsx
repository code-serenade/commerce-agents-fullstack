import { CopyableText } from "@workspace/app-kit"
import { Badge, Progress } from "@workspace/ui-components"
import type { DataTableColumn } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import { mapApiStatusToLabel } from "./status"
import type { TeamUsageRow } from "../types"

function formatOptionalNumber(value: number | null) {
  return typeof value === "number" ? value.toLocaleString() : "-"
}

function formatDateTime(value: string | null) {
  if (!value) return "-"
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return "-"

  return date.toLocaleString()
}

function usagePercent(row: TeamUsageRow) {
  if (row.question_limit <= 0) {
    return row.question_used > 0 ? 100 : 0
  }

  return Math.min(100, Math.round((row.question_used / row.question_limit) * 100))
}

export function buildTeamUsageColumns(
  t: TFunction
): DataTableColumn<TeamUsageRow>[] {
  return [
    {
      key: "school_name",
      header: t("admin.usage.teamUsage.table.columns.schoolName"),
      width: 180,
      renderCell: (row) => (
        <CopyableText value={row.school_name} textClassName="text-sm">
          {row.school_name}
        </CopyableText>
      ),
    },
    {
      key: "team_name",
      header: t("admin.usage.teamUsage.table.columns.teamName"),
      width: 180,
      renderCell: (row) => (
        <span className="font-medium">{row.team_name}</span>
      ),
    },
    {
      key: "member_count",
      header: t("admin.usage.teamUsage.table.columns.memberCount"),
      width: 96,
      sortable: true,
      renderCell: (row) => row.member_count.toLocaleString(),
    },
    {
      key: "question_used",
      header: t("admin.usage.teamUsage.table.columns.questionUsed"),
      width: 120,
      sortable: true,
      renderCell: (row) => row.question_used.toLocaleString(),
    },
    {
      key: "question_limit",
      header: t("admin.usage.teamUsage.table.columns.questionLimit"),
      width: 120,
      renderCell: (row) => row.question_limit.toLocaleString(),
    },
    {
      key: "question_remaining",
      header: t("admin.usage.teamUsage.table.columns.questionRemaining"),
      width: 120,
      sortable: true,
      renderCell: (row) => (
        <span
          className={
            row.question_remaining <= 0
              ? "font-medium text-destructive"
              : "text-foreground"
          }
        >
          {row.question_remaining.toLocaleString()}
        </span>
      ),
    },
    {
      key: "question_progress",
      header: t("admin.usage.teamUsage.table.columns.questionProgress"),
      width: 160,
      renderCell: (row) => {
        const percent = usagePercent(row)

        return (
          <div className="grid min-w-32 gap-1">
            <Progress value={percent} />
            <span className="text-xs text-(--app-muted-text)">
              {percent}%
            </span>
          </div>
        )
      },
    },
    {
      key: "token_used",
      header: t("admin.usage.teamUsage.table.columns.tokenUsed"),
      width: 120,
      renderCell: (row) => row.token_used.toLocaleString(),
    },
    {
      key: "token_limit",
      header: t("admin.usage.teamUsage.table.columns.tokenLimit"),
      width: 120,
      renderCell: (row) => formatOptionalNumber(row.token_limit),
    },
    {
      key: "status",
      header: t("admin.usage.teamUsage.table.columns.status"),
      width: 100,
      renderCell: (row) => (
        <Badge variant={row.status === "enabled" ? "default" : "outline"}>
          {mapApiStatusToLabel(row.status, t)}
        </Badge>
      ),
    },
    {
      key: "remark",
      header: t("admin.usage.teamUsage.table.columns.remark"),
      width: 220,
      renderCell: (row) =>
        row.remark?.trim() ? (
          row.remark
        ) : (
          <span className="text-sm text-(--app-muted-text)">
            {t("admin.usage.teamUsage.table.empty.remark")}
          </span>
        ),
    },
    {
      key: "lastUsedAt",
      header: t("admin.usage.teamUsage.table.columns.lastUsedAt"),
      width: 180,
      sortable: true,
      renderCell: (row) => formatDateTime(row.lastUsedAt),
    },
  ]
}
