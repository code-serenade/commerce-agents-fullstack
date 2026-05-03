import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import type { DataTableRowActionsConfig } from "@workspace/app-kit"
import { toast } from "@workspace/ui-components"
import { deleteAppRoleApi } from "@/api"
import type { RoleRow } from "../types"

export function useRoleRowActions(
  invalidateRoles: () => Promise<unknown>
): DataTableRowActionsConfig<RoleRow> {
  const { t } = useTranslation()

  return useMemo(
    () => ({
      columnWidth: 72,
      edit: false,
      delete: {
        label: t("admin.access.appRoles.table.actions.delete.label"),
        title: (row) =>
          t("admin.access.appRoles.table.actions.delete.title", {
            name: row.name,
          }),
        description: (row) =>
          t("admin.access.appRoles.table.actions.delete.description", {
            code: row.code,
            name: row.name,
          }),
        confirmLabel: t("admin.access.appRoles.table.actions.delete.confirm"),
        onConfirm: async (row) => {
          await deleteAppRoleApi(row.id)
          await invalidateRoles()
          toast.success(
            t("admin.access.appRoles.table.actions.delete.success")
          )
        },
      },
    }),
    [invalidateRoles, t]
  )
}
