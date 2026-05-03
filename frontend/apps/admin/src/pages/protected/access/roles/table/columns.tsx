import { CopyableText } from "@workspace/app-kit"
import type { DataTableColumn } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { RoleRow } from "../types"

export function buildRoleColumns(t: TFunction): DataTableColumn<RoleRow>[] {
  return [
    {
      key: "id",
      header: t("admin.access.roles.table.columns.id"),
      width: 90,
      sortable: true,
      renderCell: (row) => row.id,
    },
    {
      key: "name",
      header: t("admin.access.roles.table.columns.name"),
      width: 180,
      sortable: true,
      renderCell: (row) => <span className="font-medium">{row.name}</span>,
    },
    {
      key: "code",
      header: t("admin.access.roles.table.columns.code"),
      width: 180,
      sortable: true,
      renderCell: (row) => (
        <CopyableText
          value={row.code}
          textClassName="font-mono text-sm"
        >
          {row.code}
        </CopyableText>
      ),
    },
  ]
}
