import type { RolePermissionTreeNode } from "@/api"

export const appRolePermissionsRolesQueryKey = [
  "admin",
  "app-role-permissions",
  "roles",
] as const

export const emptyPermissionTree: RolePermissionTreeNode[] = []

export function appRolePermissionQueryKey(roleId: string) {
  return ["admin", "app-role-permissions", roleId] as const
}
