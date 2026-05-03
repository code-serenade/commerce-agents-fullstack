import { forwardRef, useImperativeHandle, useState } from "react"
import { useTranslation } from "react-i18next"
import { Input } from "@workspace/ui-components"
import type { AdminUserRow } from "../types"

export interface EditAdminUserDialogContentHandle {
  getRemark: () => string
}

export const EditAdminUserDialogContent =
  forwardRef<EditAdminUserDialogContentHandle, { row: AdminUserRow }>(
    function EditAdminUserDialogContent({ row }, ref) {
      const { t } = useTranslation()
      const [remark, setRemark] = useState(row.remark ?? "")

      useImperativeHandle(
        ref,
        () => ({
          getRemark: () => remark,
        }),
        [remark]
      )

      return (
        <div className="grid gap-4 py-2">
          <div className="grid gap-2">
            <span className="text-sm font-medium">
              {t("admin.accounts.adminUsers.edit.fields.id")}
            </span>
            <span className="font-mono text-sm text-muted-foreground">
              {row.display_id}
            </span>
          </div>
          <div className="grid gap-2">
            <span className="text-sm font-medium">
              {t("admin.accounts.adminUsers.edit.fields.displayName")}
            </span>
            <span className="text-sm text-foreground">{row.display_name}</span>
          </div>
          <div className="grid gap-2">
            <span className="text-sm font-medium">
              {t("admin.accounts.adminUsers.edit.fields.remark")}
            </span>
            <Input
              value={remark}
              onValueChange={setRemark}
              placeholder={t("admin.accounts.adminUsers.edit.remarkPlaceholder")}
            />
          </div>
        </div>
      )
    }
  )
