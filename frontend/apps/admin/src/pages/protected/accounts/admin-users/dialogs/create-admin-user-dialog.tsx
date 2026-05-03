import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import type { DataTableInsertActionConfig } from "@workspace/app-kit"
import { Input, Select } from "@workspace/ui-components"
import { createAdminUserApi, registerApi } from "@/api"

type CreateAdminUserMode = "existing-account" | "new-account"

export function useCreateAdminUserInsertAction(
  invalidateAdminUsers: () => Promise<unknown>
): DataTableInsertActionConfig {
  const { t } = useTranslation()
  const [createMode, setCreateMode] =
    useState<CreateAdminUserMode>("existing-account")
  const [existingIdentifier, setExistingIdentifier] = useState("")
  const [newUsername, setNewUsername] = useState("")
  const [newPassword, setNewPassword] = useState("")
  const [newDisplayName, setNewDisplayName] = useState("")
  const [newEmail, setNewEmail] = useState("")
  const [draftRemark, setDraftRemark] = useState("")

  const createModeOptions = useMemo(
    () =>
      [
        {
          label: t("admin.accounts.adminUsers.create.mode.existing"),
          value: "existing-account",
        },
        {
          label: t("admin.accounts.adminUsers.create.mode.new"),
          value: "new-account",
        },
      ] as const,
    [t]
  )

  return useMemo(
    () => ({
      label: t("admin.accounts.adminUsers.create.label"),
      title: t("admin.accounts.adminUsers.create.title"),
      description: t("admin.accounts.adminUsers.create.description"),
      renderContent: () => (
        <div className="grid gap-4 py-2">
          <div className="grid gap-2">
            <span className="text-sm font-medium">
              {t("admin.accounts.adminUsers.create.mode.label")}
            </span>
            <Select
              value={createMode}
              onValueChange={(value) =>
                setCreateMode(value as CreateAdminUserMode)
              }
              list={createModeOptions.map((option) => ({
                label: option.label,
                value: option.value,
              }))}
            />
          </div>

          {createMode === "existing-account" ? (
            <>
              <div className="grid gap-2">
                <span className="text-sm font-medium">
                  {t("admin.accounts.adminUsers.create.fields.account.label")}
                </span>
                <Input
                  value={existingIdentifier}
                  onValueChange={setExistingIdentifier}
                  placeholder={t(
                    "admin.accounts.adminUsers.create.fields.account.placeholder"
                  )}
                />
              </div>
            </>
          ) : (
            <>
              <div className="grid gap-2">
                <span className="text-sm font-medium">
                  {t("admin.accounts.adminUsers.create.fields.username.label")}
                </span>
                <Input
                  value={newUsername}
                  onValueChange={setNewUsername}
                  placeholder={t(
                    "admin.accounts.adminUsers.create.fields.username.placeholder"
                  )}
                />
              </div>
              <div className="grid gap-2">
                <span className="text-sm font-medium">
                  {t("admin.accounts.adminUsers.create.fields.password.label")}
                </span>
                <Input
                  value={newPassword}
                  onValueChange={setNewPassword}
                  placeholder={t(
                    "admin.accounts.adminUsers.create.fields.password.placeholder"
                  )}
                  type="password"
                />
              </div>
              <div className="grid gap-2">
                <span className="text-sm font-medium">
                  {t(
                    "admin.accounts.adminUsers.create.fields.displayName.label"
                  )}
                </span>
                <Input
                  value={newDisplayName}
                  onValueChange={setNewDisplayName}
                  placeholder={t(
                    "admin.accounts.adminUsers.create.fields.displayName.placeholder"
                  )}
                />
              </div>
              <div className="grid gap-2">
                <span className="text-sm font-medium">
                  {t("admin.accounts.adminUsers.create.fields.email.label")}
                </span>
                <Input
                  value={newEmail}
                  onValueChange={setNewEmail}
                  placeholder={t(
                    "admin.accounts.adminUsers.create.fields.email.placeholder"
                  )}
                  type="email"
                />
              </div>
            </>
          )}

          <div className="grid gap-2">
            <span className="text-sm font-medium">
              {t("admin.accounts.adminUsers.create.fields.remark.label")}
            </span>
            <Input
              value={draftRemark}
              onValueChange={setDraftRemark}
              placeholder={t(
                "admin.accounts.adminUsers.create.fields.remark.placeholder"
              )}
            />
          </div>
        </div>
      ),
      onConfirm: async () => {
        const trimmedRemark = draftRemark.trim()

        if (createMode === "existing-account") {
          const identifier = existingIdentifier.trim()

          if (!identifier) {
            throw new Error(
              t("admin.accounts.adminUsers.create.errors.identifierRequired")
            )
          }

          await createAdminUserApi({
            identifier,
            remark: trimmedRemark || null,
          })
        } else {
          const username = newUsername.trim()
          const password = newPassword.trim()
          const displayName = newDisplayName.trim()
          const email = newEmail.trim()

          if (!username) {
            throw new Error(
              t("admin.accounts.adminUsers.create.errors.usernameRequired")
            )
          }
          if (!password) {
            throw new Error(
              t("admin.accounts.adminUsers.create.errors.passwordRequired")
            )
          }
          if (!displayName) {
            throw new Error(
              t("admin.accounts.adminUsers.create.errors.displayNameRequired")
            )
          }

          await registerApi({
            username,
            password,
            display_name: displayName,
            email: email || undefined,
          })

          await createAdminUserApi({
            identifier: username,
            remark: trimmedRemark || null,
          })
        }

        setCreateMode("existing-account")
        setExistingIdentifier("")
        setNewUsername("")
        setNewPassword("")
        setNewDisplayName("")
        setNewEmail("")
        setDraftRemark("")
        await invalidateAdminUsers()
      },
    }),
    [
      createMode,
      createModeOptions,
      draftRemark,
      existingIdentifier,
      invalidateAdminUsers,
      newDisplayName,
      newEmail,
      newPassword,
      newUsername,
      t,
    ]
  )
}
