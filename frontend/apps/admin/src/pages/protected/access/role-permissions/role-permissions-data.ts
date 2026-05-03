import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useTranslation } from "react-i18next"
import { toast } from "@workspace/ui-components"
import {
  listRolePermissionsApi,
  listRolesApi,
  updateRolePermissionsApi,
} from "@/api"
import {
  rolePermissionQueryKey,
  rolePermissionsRolesQueryKey,
} from "./constants"
import { collectCheckedPermissionIds } from "./tree/logic"

export function useRolePermissionsData({
  roleId,
  setSelectedPermissionIdsForRole,
}: {
  roleId: string
  setSelectedPermissionIdsForRole: (roleId: string, ids: string[]) => void
}) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const rolesQuery = useQuery({
    queryKey: rolePermissionsRolesQueryKey,
    queryFn: listRolesApi,
  })

  const roles = rolesQuery.data ?? []
  const resolvedRoleId = roleId || (roles[0] ? String(roles[0].id) : "")

  const rolePermissionsQuery = useQuery({
    queryKey: rolePermissionQueryKey(resolvedRoleId),
    queryFn: () => listRolePermissionsApi(Number(resolvedRoleId)),
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
      updateRolePermissionsApi(Number(nextRoleId), {
        permissionIds: selectedPermissionIds.map((id) => Number(id)),
      }),
    onSuccess: (tree, variables) => {
      queryClient.setQueryData(rolePermissionQueryKey(variables.nextRoleId), tree)
      setSelectedPermissionIdsForRole(
        variables.nextRoleId,
        collectCheckedPermissionIds(tree)
      )
      toast.success(t("admin.access.rolePermissions.editor.success"))
    },
  })

  return {
    resolvedRoleId,
    rolesQuery,
    rolePermissionsQuery,
    updateRolePermissionsMutation,
  }
}
