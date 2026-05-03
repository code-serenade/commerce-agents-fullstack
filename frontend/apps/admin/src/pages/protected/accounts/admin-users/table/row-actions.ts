import { createElement, useMemo, useRef } from "react"
import { useTranslation } from "react-i18next"
import type { DataTableRowActionsConfig } from "@workspace/app-kit"
import { toast } from "@workspace/ui-components"
import { deleteAdminUserApi, updateAdminUserApi } from "@/api"
import {
  EditAdminUserDialogContent,
  type EditAdminUserDialogContentHandle,
} from "../dialogs/edit-admin-user-dialog"
import type { AdminUserRow } from "../types"

export function useAdminUserRowActions(
  invalidateAdminUsers: () => Promise<unknown>,
  onEditRoles: (row: AdminUserRow) => void
): DataTableRowActionsConfig<AdminUserRow> {
  const { t } = useTranslation()
  const editDialogHandlesRef = useRef(
    new Map<string, EditAdminUserDialogContentHandle | null>()
  )

  return useMemo(() => {
    const setEditDialogHandle = (
      userId: string,
      handle: EditAdminUserDialogContentHandle | null
    ) => {
      if (handle) {
        editDialogHandlesRef.current.set(userId, handle)
        return
      }

      editDialogHandlesRef.current.delete(userId)
    }

    return {
      columnWidth: 118,
      edit: {
        label: t("admin.accounts.adminUsers.table.actions.edit.label"),
        title: (row) =>
          t("admin.accounts.adminUsers.table.actions.edit.title", {
            id: row.display_id,
          }),
        description: t(
          "admin.accounts.adminUsers.table.actions.edit.description"
        ),
        renderContent: ({ row }) =>
          createElement(EditAdminUserDialogContent, {
            row,
            ref: (handle) => setEditDialogHandle(row.user_id, handle),
          }),
        onConfirm: async (row) => {
          const remark =
            editDialogHandlesRef.current.get(row.user_id)?.getRemark() ??
            row.remark ??
            ""

          await updateAdminUserApi(row.user_id, {
            remark: remark.trim() || null,
            status: row.status,
          })
          await invalidateAdminUsers()
          toast.success(
            t("admin.accounts.adminUsers.table.actions.edit.success")
          )
        },
      },
      delete: {
        label: t("admin.accounts.adminUsers.table.actions.delete.label"),
        title: (row) =>
          t("admin.accounts.adminUsers.table.actions.delete.title", {
            id: row.display_id,
          }),
        description: (row) =>
          t("admin.accounts.adminUsers.table.actions.delete.description", {
            id: row.display_id,
            name: row.display_name,
          }),
        confirmLabel: t(
          "admin.accounts.adminUsers.table.actions.delete.confirm"
        ),
        onConfirm: async (row) => {
          await deleteAdminUserApi(row.user_id)
          await invalidateAdminUsers()
          toast.success(
            t("admin.accounts.adminUsers.table.actions.delete.success")
          )
        },
      },
      moreItems: [
        {
          key: "toggle-status",
          label: (row) =>
            row.status === "enabled"
              ? t("admin.accounts.adminUsers.table.actions.toggle.disable")
              : t("admin.accounts.adminUsers.table.actions.toggle.enable"),
          onClick: async (row) => {
            const nextStatus = row.status === "enabled" ? "disabled" : "enabled"

            await updateAdminUserApi(row.user_id, {
              remark: row.remark,
              status: nextStatus,
            })
            await invalidateAdminUsers()
            toast.success(
              nextStatus === "enabled"
                ? t(
                    "admin.accounts.adminUsers.table.actions.toggle.enabledSuccess"
                  )
                : t(
                    "admin.accounts.adminUsers.table.actions.toggle.disabledSuccess"
                  )
            )
          },
        },
        {
          key: "assign-role",
          label: t("admin.accounts.adminUsers.table.actions.editRoles"),
          onClick: onEditRoles,
        },
      ],
    }
  }, [invalidateAdminUsers, onEditRoles, t])
}
