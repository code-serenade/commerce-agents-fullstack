import { CopyableText } from "@workspace/app-kit"
import { Badge } from "@workspace/ui-components/stable/badge"
import type { DataTableColumn } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import { mapApiStatusToLabel } from "./status"
import type { AppUserRow } from "../types"

export function buildAppUserColumns(
  t: TFunction
): DataTableColumn<AppUserRow>[] {
  return [
    {
      key: "display_id",
      header: t("admin.accounts.appUsers.table.columns.id"),
      width: 180,
      renderCell: (row) => (
        <CopyableText
          value={row.display_id}
          textClassName="font-mono text-sm"
        >
          {row.display_id}
        </CopyableText>
      ),
    },
    {
      key: "display_name",
      header: t("admin.accounts.appUsers.table.columns.displayName"),
      width: 160,
      renderCell: (row) => (
        <span className="font-medium">{row.display_name}</span>
      ),
    },
    {
      key: "remark",
      header: t("admin.accounts.appUsers.table.columns.remark"),
      width: 220,
      renderCell: (row) =>
        row.remark?.trim() ? (
          row.remark
        ) : (
          <span className="text-sm text-(--app-muted-text)">
            {t("admin.accounts.appUsers.table.empty.remark")}
          </span>
        ),
    },
    {
      key: "status",
      header: t("admin.accounts.appUsers.table.columns.status"),
      width: 100,
      renderCell: (row) => (
        <Badge variant={row.status === "enabled" ? "default" : "outline"}>
          {mapApiStatusToLabel(row.status, t)}
        </Badge>
      ),
    },
    {
      key: "roles",
      header: t("admin.accounts.appUsers.table.columns.roles"),
      width: 180,
      renderCell: (row) =>
        row.roles.length > 0 ? (
          <div className="flex flex-wrap gap-2">
            {row.roles.map((role) => (
              <Badge key={role} variant="outline">
                {role}
              </Badge>
            ))}
          </div>
        ) : (
          <span className="text-sm text-(--app-muted-text)">
            {t("admin.accounts.appUsers.table.empty.roles")}
          </span>
        ),
    },
    {
      key: "roleCount",
      header: t("admin.accounts.appUsers.table.columns.roleCount"),
      width: 88,
      renderCell: (row) => row.roles.length,
    },
  ]
}
