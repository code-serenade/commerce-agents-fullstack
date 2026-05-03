import { forwardRef, useImperativeHandle, useState } from "react"
import { useTranslation } from "react-i18next"
import { Input } from "@workspace/ui-components"
import type { AppUserRow } from "../types"

export interface EditAppUserDialogContentHandle {
  getRemark: () => string
}

export const EditAppUserDialogContent =
  forwardRef<EditAppUserDialogContentHandle, { row: AppUserRow }>(
    function EditAppUserDialogContent({ row }, ref) {
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
              {t("admin.accounts.appUsers.edit.fields.id")}
            </span>
            <span className="font-mono text-sm text-muted-foreground">
              {row.display_id}
            </span>
          </div>
          <div className="grid gap-2">
            <span className="text-sm font-medium">
              {t("admin.accounts.appUsers.edit.fields.displayName")}
            </span>
            <span className="text-sm text-foreground">{row.display_name}</span>
          </div>
          <div className="grid gap-2">
            <span className="text-sm font-medium">
              {t("admin.accounts.appUsers.edit.fields.remark")}
            </span>
            <Input
              value={remark}
              onValueChange={setRemark}
              placeholder={t("admin.accounts.appUsers.edit.remarkPlaceholder")}
            />
          </div>
        </div>
      )
    }
  )
