import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import type { RolePermissionTreeNode } from "@/api"
import { emptyPermissionTree } from "./constants"
import { useRolePermissionsData } from "./role-permissions-data"
import {
  buildPermissionSummary,
  collectCheckedPermissionIds,
  flattenPermissionTree,
  toTreeNode,
} from "./tree/logic"
import type { SelectedPermissionIdsByRole } from "./types"

export function useRolePermissionsPage() {
  const { t } = useTranslation()
  const [activeRoleId, setActiveRoleId] = useState("")
  const [searchValue, setSearchValue] = useState("")
  const [selectedPermissionIdsByRole, setSelectedPermissionIdsByRole] =
    useState<SelectedPermissionIdsByRole>({})

  const setSelectedPermissionIdsForRole = (roleId: string, ids: string[]) => {
    setSelectedPermissionIdsByRole((current) => ({
      ...current,
      [roleId]: ids,
    }))
  }

  const {
    resolvedRoleId,
    rolesQuery,
    rolePermissionsQuery,
    updateRolePermissionsMutation,
  } = useRolePermissionsData({
    roleId: activeRoleId,
    setSelectedPermissionIdsForRole,
  })

  const roles = rolesQuery.data ?? []
  const activeRole = roles.find((role) => String(role.id) === resolvedRoleId)
  const permissionTree = rolePermissionsQuery.data ?? emptyPermissionTree
  const serverSelectedPermissionIds = useMemo(
    () => collectCheckedPermissionIds(permissionTree),
    [permissionTree]
  )
  const selectedPermissionIds =
    Object.prototype.hasOwnProperty.call(
      selectedPermissionIdsByRole,
      resolvedRoleId
    )
      ? selectedPermissionIdsByRole[resolvedRoleId]
      : serverSelectedPermissionIds
  const flatPermissions = useMemo(
    () => flattenPermissionTree(permissionTree),
    [permissionTree]
  )
  const selectedResources = useMemo(
    () =>
      selectedPermissionIds
        .map((id) =>
          flatPermissions.find((permission) => String(permission.id) === id)
        )
        .filter(
          (permission): permission is RolePermissionTreeNode =>
            Boolean(permission)
        ),
    [flatPermissions, selectedPermissionIds]
  )
  const resourceTree = useMemo(
    () => permissionTree.map((node) => toTreeNode(node, t)),
    [permissionTree, t]
  )
  const defaultExpandedIds = useMemo(
    () => permissionTree.map((node) => String(node.id)),
    [permissionTree]
  )
  const summary = useMemo(
    () => buildPermissionSummary(selectedResources),
    [selectedResources]
  )

  const handleRoleChange = (roleId: string) => {
    setActiveRoleId(roleId)
    setSearchValue("")
  }

  const handleResetSelection = () => {
    setSelectedPermissionIdsForRole(resolvedRoleId, serverSelectedPermissionIds)
  }

  const handleSave = () => {
    updateRolePermissionsMutation.mutate({
      nextRoleId: resolvedRoleId,
      selectedPermissionIds,
    })
  }

  const handleSelectedPermissionIdsChange = (ids: string[]) => {
    setSelectedPermissionIdsForRole(resolvedRoleId, ids)
  }

  return {
    activeRole,
    defaultExpandedIds,
    handleResetSelection,
    handleRoleChange,
    handleSave,
    handleSearchChange: setSearchValue,
    handleSelectedPermissionIdsChange,
    isSaving: updateRolePermissionsMutation.isPending,
    isTreeLoading: rolePermissionsQuery.isFetching,
    resolvedRoleId,
    resourceTree,
    roles,
    searchValue,
    selectedPermissionIds,
    summary,
  }
}
