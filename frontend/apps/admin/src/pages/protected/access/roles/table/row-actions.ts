import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import type { DataTableRowActionsConfig } from "@workspace/app-kit"
import { toast } from "@workspace/ui-components"
import { deleteRoleApi } from "@/api"
import type { RoleRow } from "../types"

export function useRoleRowActions(
  invalidateRoles: () => Promise<unknown>
): DataTableRowActionsConfig<RoleRow> {
  const { t } = useTranslation()

  return useMemo(
    () => ({
      columnWidth: 56,
      edit: false,
      delete: {
        label: t("admin.access.roles.table.actions.delete.label"),
        title: (row) =>
          t("admin.access.roles.table.actions.delete.title", {
            name: row.name,
          }),
        description: (row) =>
          t("admin.access.roles.table.actions.delete.description", {
            code: row.code,
            name: row.name,
          }),
        confirmLabel: t("admin.access.roles.table.actions.delete.confirm"),
        onConfirm: async (row) => {
          await deleteRoleApi(row.id)
          await invalidateRoles()
          toast.success(t("admin.access.roles.table.actions.delete.success"))
        },
      },
    }),
    [invalidateRoles, t]
  )
}
