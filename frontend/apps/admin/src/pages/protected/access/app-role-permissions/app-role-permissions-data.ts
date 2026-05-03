import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useTranslation } from "react-i18next"
import { toast } from "@workspace/ui-components"
import {
  listAppRolePermissionsApi,
  listAppRolesApi,
  updateAppRolePermissionsApi,
} from "@/api"
import {
  appRolePermissionQueryKey,
  appRolePermissionsRolesQueryKey,
} from "./constants"
import { collectCheckedPermissionIds } from "./tree/logic"

export function useAppRolePermissionsData({
  roleId,
  setSelectedPermissionIdsForRole,
}: {
  roleId: string
  setSelectedPermissionIdsForRole: (roleId: string, ids: string[]) => void
}) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const rolesQuery = useQuery({
    queryKey: appRolePermissionsRolesQueryKey,
    queryFn: listAppRolesApi,
  })

  const roles = rolesQuery.data ?? []
  const resolvedRoleId = roleId || (roles[0] ? String(roles[0].id) : "")

  const rolePermissionsQuery = useQuery({
    queryKey: appRolePermissionQueryKey(resolvedRoleId),
    queryFn: () => listAppRolePermissionsApi(Number(resolvedRoleId)),
    enabled: resolvedRoleId.length > 0,
  })

  const updateRolePermissionsMutation = useMutation({
    mutationFn: async ({
      nextRoleId,
      selectedPermissionIds,
    }: {
      nextRoleId: string
      selectedPermissionIds: readonly string[]
    }) =>
      updateAppRolePermissionsApi(Number(nextRoleId), {
        permissionIds: selectedPermissionIds.map((id) => Number(id)),
      }),
    onSuccess: (tree, variables) => {
      queryClient.setQueryData(
        appRolePermissionQueryKey(variables.nextRoleId),
        tree
      )
      setSelectedPermissionIdsForRole(
        variables.nextRoleId,
        collectCheckedPermissionIds(tree)
      )
      toast.success(t("admin.access.appRolePermissions.editor.success"))
    },
  })

  return {
    resolvedRoleId,
    rolesQuery,
    rolePermissionsQuery,
    updateRolePermissionsMutation,
  }
}
