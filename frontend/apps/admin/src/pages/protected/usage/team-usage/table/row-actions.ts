import { createElement, useMemo, useRef } from "react"
import { useTranslation } from "react-i18next"
import type { DataTableRowActionsConfig } from "@workspace/app-kit"
import { toast } from "@workspace/ui-components"
import { updateTeamUsageQuotaApi } from "@/api"
import {
  EditTeamQuotaDialogContent,
  type EditTeamQuotaDialogContentHandle,
} from "../dialogs/edit-team-quota-dialog"
import type { TeamUsageRow } from "../types"

export function useTeamUsageRowActions(
  invalidateTeamUsage: () => Promise<unknown>,
  onViewMembers: (row: TeamUsageRow) => void
): DataTableRowActionsConfig<TeamUsageRow> {
  const { t } = useTranslation()
  const editDialogHandlesRef = useRef(
    new Map<string, EditTeamQuotaDialogContentHandle | null>()
  )

  return useMemo(() => {
    const setEditDialogHandle = (
      teamId: string,
      handle: EditTeamQuotaDialogContentHandle | null
    ) => {
      if (handle) {
        editDialogHandlesRef.current.set(teamId, handle)
        return
      }

      editDialogHandlesRef.current.delete(teamId)
    }

    return {
      columnWidth: 118,
      edit: {
        label: t("admin.usage.teamUsage.table.actions.edit.label"),
        title: (row) =>
          t("admin.usage.teamUsage.table.actions.edit.title", {
            team: row.team_name,
          }),
        description: t("admin.usage.teamUsage.table.actions.edit.description"),
        renderContent: ({ row }) =>
          createElement(EditTeamQuotaDialogContent, {
            row,
            ref: (handle) => setEditDialogHandle(row.id, handle),
          }),
        onConfirm: async (row) => {
          const value = editDialogHandlesRef.current.get(row.id)?.getValue()

          await updateTeamUsageQuotaApi(row.id, {
            questionLimit: value?.questionLimit ?? row.question_limit,
            tokenLimit: value?.tokenLimit ?? row.token_limit,
            status: value?.status ?? row.status,
            remark: value?.remark ?? row.remark,
          })
          await invalidateTeamUsage()
          toast.success(t("admin.usage.teamUsage.table.actions.edit.success"))
        },
      },
      delete: false,
      moreItems: [
        {
          key: "toggle-status",
          label: (row) =>
            row.status === "enabled"
              ? t("admin.usage.teamUsage.table.actions.toggle.disable")
              : t("admin.usage.teamUsage.table.actions.toggle.enable"),
          onClick: async (row) => {
            const nextStatus = row.status === "enabled" ? "disabled" : "enabled"

            await updateTeamUsageQuotaApi(row.id, {
              questionLimit: row.question_limit,
              tokenLimit: row.token_limit,
              status: nextStatus,
              remark: row.remark,
            })
            await invalidateTeamUsage()
            toast.success(
              nextStatus === "enabled"
                ? t(
                    "admin.usage.teamUsage.table.actions.toggle.enabledSuccess"
                  )
                : t(
                    "admin.usage.teamUsage.table.actions.toggle.disabledSuccess"
                  )
            )
          },
        },
        {
          key: "members",
          label: t("admin.usage.teamUsage.table.actions.members"),
          onClick: onViewMembers,
        },
      ],
    }
  }, [invalidateTeamUsage, onViewMembers, t])
}
