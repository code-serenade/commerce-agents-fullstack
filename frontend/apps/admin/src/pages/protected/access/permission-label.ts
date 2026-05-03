import type { TFunction } from "i18next"

export interface PermissionLabelSource {
  code?: string | null
  name: string
}

const permissionCodeByName: Record<string, string> = {
  "Access Control": "access_control",
  "Admin Users": "accounts:admin_users",
  "App Role Permissions": "access_control:app_role_permissions",
  "App Roles": "access_control:app_roles",
  "App Users": "accounts:app_users",
  "Role Permissions": "access_control:role_permissions",
  Accounts: "accounts",
  Dashboard: "dashboard",
  Roles: "access_control:roles",
  Profile: "profile",
  "Update Profile": "profile:update",
  "View Profile": "profile:view",
}

export function getPermissionLabel(
  t: TFunction,
  permission: PermissionLabelSource
) {
  const code = resolvePermissionCode(permission)
  if (!code) return permission.name

  const key = `admin.permissions.${code.replaceAll(":", ".")}.label`
  const label = t(key)

  return label === key ? permission.name : label
}

export function getPermissionSearchText(
  t: TFunction,
  permission: PermissionLabelSource
) {
  const label = getPermissionLabel(t, permission)
  const code = resolvePermissionCode(permission)
  return [label, code, permission.name].filter(Boolean).join(" ")
}

function resolvePermissionCode(permission: PermissionLabelSource) {
  const code = permission.code?.trim()
  if (code) return code

  return permissionCodeByName[permission.name.trim()] ?? null
}
