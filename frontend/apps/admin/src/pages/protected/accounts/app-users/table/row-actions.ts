import { createElement, useMemo, useRef } from "react"
import { useTranslation } from "react-i18next"
import type { DataTableRowActionsConfig } from "@workspace/app-kit"
import { toast } from "@workspace/ui-components"
import { deleteAppUserApi, updateAppUserApi } from "@/api"
import {
  EditAppUserDialogContent,
  type EditAppUserDialogContentHandle,
} from "../dialogs/edit-app-user-dialog"
import type { AppUserRow } from "../types"

export function useAppUserRowActions(
  invalidateAppUsers: () => Promise<unknown>,
  onEditRoles: (row: AppUserRow) => void
): DataTableRowActionsConfig<AppUserRow> {
  const { t } = useTranslation()
  const editDialogHandlesRef = useRef(
    new Map<string, EditAppUserDialogContentHandle | null>()
  )

  return useMemo(() => {
    const setEditDialogHandle = (
      userId: string,
      handle: EditAppUserDialogContentHandle | null
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
        label: t("admin.accounts.appUsers.table.actions.edit.label"),
        title: (row) =>
          t("admin.accounts.appUsers.table.actions.edit.title", {
            id: row.display_id,
          }),
        description: t(
          "admin.accounts.appUsers.table.actions.edit.description"
        ),
        renderContent: ({ row }) =>
          createElement(EditAppUserDialogContent, {
            row,
            ref: (handle) => setEditDialogHandle(row.user_id, handle),
          }),
        onConfirm: async (row) => {
          const remark =
            editDialogHandlesRef.current.get(row.user_id)?.getRemark() ??
            row.remark ??
            ""

          await updateAppUserApi(row.user_id, {
            remark: remark.trim() || null,
            status: row.status,
          })
          await invalidateAppUsers()
          toast.success(t("admin.accounts.appUsers.table.actions.edit.success"))
        },
      },
      delete: {
        label: t("admin.accounts.appUsers.table.actions.delete.label"),
        title: (row) =>
          t("admin.accounts.appUsers.table.actions.delete.title", {
            id: row.display_id,
          }),
        description: (row) =>
          t("admin.accounts.appUsers.table.actions.delete.description", {
            id: row.display_id,
            name: row.display_name,
          }),
        confirmLabel: t(
          "admin.accounts.appUsers.table.actions.delete.confirm"
        ),
        onConfirm: async (row) => {
          await deleteAppUserApi(row.user_id)
          await invalidateAppUsers()
          toast.success(
            t("admin.accounts.appUsers.table.actions.delete.success")
          )
        },
      },
      moreItems: [
        {
          key: "toggle-status",
          label: (row) =>
            row.status === "enabled"
              ? t("admin.accounts.appUsers.table.actions.toggle.disable")
              : t("admin.accounts.appUsers.table.actions.toggle.enable"),
          onClick: async (row) => {
            const nextStatus = row.status === "enabled" ? "disabled" : "enabled"

            await updateAppUserApi(row.user_id, {
              remark: row.remark,
              status: nextStatus,
            })
            await invalidateAppUsers()
            toast.success(
              nextStatus === "enabled"
                ? t(
                    "admin.accounts.appUsers.table.actions.toggle.enabledSuccess"
                  )
                : t(
                    "admin.accounts.appUsers.table.actions.toggle.disabledSuccess"
                  )
            )
          },
        },
        {
          key: "assign-role",
          label: t("admin.accounts.appUsers.table.actions.editRoles"),
          onClick: onEditRoles,
        },
      ],
    }
  }, [invalidateAppUsers, onEditRoles, t])
}
