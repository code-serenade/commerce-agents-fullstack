import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import type { DataTableInsertActionConfig } from "@workspace/app-kit"
import { Input, toast } from "@workspace/ui-components"
import { createRoleApi } from "@/api"

export function useCreateRoleInsertAction(
  invalidateRoles: () => Promise<unknown>
): DataTableInsertActionConfig {
  const { t } = useTranslation()
  const [name, setName] = useState("")
  const [code, setCode] = useState("")

  return useMemo(
    () => ({
      label: t("admin.access.roles.create.label"),
      title: t("admin.access.roles.create.title"),
      description: t("admin.access.roles.create.description"),
      renderContent: () => (
        <div className="grid gap-4 py-2">
          <div className="grid gap-2">
            <span className="text-sm font-medium">
              {t("admin.access.roles.create.fields.name.label")}
            </span>
            <Input
              value={name}
              onValueChange={setName}
              placeholder={t(
                "admin.access.roles.create.fields.name.placeholder"
              )}
            />
          </div>
          <div className="grid gap-2">
            <span className="text-sm font-medium">
              {t("admin.access.roles.create.fields.code.label")}
            </span>
            <Input
              value={code}
              onValueChange={setCode}
              placeholder={t(
                "admin.access.roles.create.fields.code.placeholder"
              )}
            />
          </div>
        </div>
      ),
      onConfirm: async () => {
        const trimmedName = name.trim()
        const trimmedCode = code.trim()

        if (!trimmedName) {
          throw new Error(t("admin.access.roles.create.errors.nameRequired"))
        }
        if (!trimmedCode) {
          throw new Error(t("admin.access.roles.create.errors.codeRequired"))
        }

        await createRoleApi({
          name: trimmedName,
          code: trimmedCode,
        })

        setName("")
        setCode("")
        await invalidateRoles()
        toast.success(t("admin.access.roles.create.success"))
      },
    }),
    [code, invalidateRoles, name, t]
  )
}
