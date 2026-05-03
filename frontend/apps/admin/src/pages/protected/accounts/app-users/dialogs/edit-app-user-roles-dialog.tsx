import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import type { QueryClient } from "@tanstack/react-query"
import type { TFunction } from "i18next"
import { toast } from "@workspace/ui-components"
import { Button } from "@workspace/ui-components/stable/button"
import { Checkbox } from "@workspace/ui-components/stable/checkbox"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@workspace/ui-components/stable/dialog"
import {
  listAppUserRolesApi,
  updateAppUserRolesApi,
  type AppUserRoleOptionResponse,
} from "@/api"
import { appUserMetricsQueryKey } from "../constants"
import type { AppUserRow } from "../types"

const userRoleOptionsQueryKey = (userId: string) =>
  ["admin", "app-user-roles", userId] as const

export function EditAppUserRolesDialog({
  onSaved,
  onOpenChange,
  open,
  row,
}: {
  onSaved: () => void
  onOpenChange: (open: boolean) => void
  open: boolean
  row: AppUserRow | null
}) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const userId = row?.user_id ?? ""
  const displayName =
    row?.display_name?.trim() ||
    t("admin.accounts.appUsers.data.fallbackDisplayName")

  const rolesQuery = useQuery({
    queryKey: userRoleOptionsQueryKey(userId),
    queryFn: () => listAppUserRolesApi(userId),
    enabled: open && Boolean(userId),
  })

  const allRoleOptions = useMemo(() => rolesQuery.data ?? [], [rolesQuery.data])
  const visibleRoleOptions = useMemo(
    () => allRoleOptions.filter((role) => role.code !== "root"),
    [allRoleOptions]
  )
  const hiddenSelectedRoleIds = useMemo(
    () =>
      allRoleOptions
        .filter((role) => role.code === "root" && role.checked)
        .map((role) => role.id),
    [allRoleOptions]
  )

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {t("admin.accounts.appUsers.rolesDialog.title", {
              name: row ? ` ${displayName}` : "",
            })}
          </DialogTitle>
          {row ? (
            <div className="font-mono text-xs text-muted-foreground">
              {t("admin.accounts.appUsers.rolesDialog.id", {
                id: row.display_id,
              })}
            </div>
          ) : null}
          <DialogDescription>
            {t("admin.accounts.appUsers.rolesDialog.description")}
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-2">
          {rolesQuery.isFetching ? (
            <div className="max-h-80 overflow-auto">
              <div className="px-3 py-4 text-sm text-muted-foreground">
                {t("admin.accounts.appUsers.rolesDialog.loading")}
              </div>
            </div>
          ) : visibleRoleOptions.length > 0 ? (
            <EditAppUserRolesForm
              key={`${userId}:${rolesQuery.dataUpdatedAt}`}
              hiddenSelectedRoleIds={hiddenSelectedRoleIds}
              onClose={() => onOpenChange(false)}
              onSaved={onSaved}
              queryClient={queryClient}
              roleOptions={visibleRoleOptions}
              t={t}
              userId={userId}
            />
          ) : (
            <div className="max-h-80 overflow-auto">
              <div className="px-3 py-4 text-sm text-muted-foreground">
                {t("admin.accounts.appUsers.rolesDialog.empty")}
              </div>
            </div>
          )}
        </div>

        {rolesQuery.isFetching || visibleRoleOptions.length === 0 ? (
          <DialogFooter>
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              {t("admin.accounts.appUsers.rolesDialog.cancel")}
            </Button>
          </DialogFooter>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}

function EditAppUserRolesForm({
  hiddenSelectedRoleIds,
  onClose,
  onSaved,
  queryClient,
  roleOptions,
  t,
  userId,
}: {
  hiddenSelectedRoleIds: number[]
  onClose: () => void
  onSaved: () => void
  queryClient: QueryClient
  roleOptions: AppUserRoleOptionResponse[]
  t: TFunction
  userId: string
}) {
  const serverSelectedRoleIds = useMemo(
    () => roleOptions.filter((role) => role.checked).map((role) => role.id),
    [roleOptions]
  )
  const [selectedRoleIds, setSelectedRoleIds] = useState<number[]>(
    serverSelectedRoleIds
  )

  const updateRolesMutation = useMutation({
    mutationFn: () =>
      updateAppUserRolesApi(userId, {
        roleIds: [...hiddenSelectedRoleIds, ...selectedRoleIds],
      }),
    onSuccess: async (roles) => {
      queryClient.setQueryData(userRoleOptionsQueryKey(userId), roles)
      await queryClient.refetchQueries({
        queryKey: appUserMetricsQueryKey,
      })
      onSaved()
      toast.success(t("admin.accounts.appUsers.rolesDialog.success"))
      onClose()
    },
  })

  const toggleRole = (role: AppUserRoleOptionResponse, checked: boolean) => {
    setSelectedRoleIds((current) => {
      if (checked) {
        return current.includes(role.id) ? current : [...current, role.id]
      }

      return current.filter((roleId) => roleId !== role.id)
    })
  }

  const handleReset = () => {
    setSelectedRoleIds(serverSelectedRoleIds)
  }

  const handleSave = () => {
    updateRolesMutation.mutate()
  }

  return (
    <>
      <div className="grid max-h-80 grid-cols-2 gap-2 overflow-auto">
        {roleOptions.map((role) => {
          const checked = selectedRoleIds.includes(role.id)

          return (
            <label
              key={role.id}
              className="flex min-w-0 cursor-pointer items-start gap-2.5 rounded-lg px-1 py-2 transition-colors hover:bg-(--app-muted-bg)"
            >
              <Checkbox
                checked={checked}
                disabled={updateRolesMutation.isPending}
                onCheckedChange={(value) => toggleRole(role, value === true)}
              />
              <span className="grid min-w-0 flex-1 gap-1">
                <span className="truncate text-sm font-medium">
                  {role.name}
                </span>
                <span className="font-mono text-xs text-muted-foreground">
                  {role.code}
                </span>
              </span>
            </label>
          )
        })}
      </div>

      <DialogFooter>
        <Button
          variant="outline"
          onClick={onClose}
          disabled={updateRolesMutation.isPending}
        >
          {t("admin.accounts.appUsers.rolesDialog.cancel")}
        </Button>
        <Button
          variant="outline"
          onClick={handleReset}
          disabled={updateRolesMutation.isPending}
        >
          {t("admin.accounts.appUsers.rolesDialog.reset")}
        </Button>
        <Button
          variant="primary"
          onClick={handleSave}
          disabled={updateRolesMutation.isPending}
        >
          {t("admin.accounts.appUsers.rolesDialog.save")}
        </Button>
      </DialogFooter>
    </>
  )
}
