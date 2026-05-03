import type { RolePermissionTreeNode } from "@/api"

export const rolePermissionsRolesQueryKey = [
  "admin",
  "role-permissions",
  "roles",
] as const

export const emptyPermissionTree: RolePermissionTreeNode[] = []

export function rolePermissionQueryKey(roleId: string) {
  return ["admin", "role-permissions", roleId] as const
}
