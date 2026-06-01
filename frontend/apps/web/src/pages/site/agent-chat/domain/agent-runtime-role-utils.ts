export const MAX_SELECTED_RUNTIME_ROLES = 6

export function preferredRuntimePrimaryRole(roles: string[]) {
  const nonChatRoles = roles.filter((role) => role !== "chat")
  if (nonChatRoles.includes("ops")) return "ops"
  return nonChatRoles[0] ?? roles[0] ?? "chat"
}

export function orderDefaultCommerceTeamRoles(roles: string[]) {
  const preferred = ["ops", "data", "creative", "service", "engineering"]
  const uniqueRoles = uniqueRuntimeRoles(roles)
  const ordered = preferred.filter((role) => uniqueRoles.includes(role))
  const rest = uniqueRoles.filter((role) => !ordered.includes(role))
  const team = [...ordered, ...rest].slice(0, 5)
  return team.length ? team : ["ops", "data", "creative"]
}

export function uniqueRuntimeRoles(roles: string[]) {
  return roles.reduce<string[]>((unique, role) => {
    const normalized = role.trim()
    if (normalized && !unique.includes(normalized)) unique.push(normalized)
    return unique
  }, [])
}

export function ensureManualRoleTeam(roles: string[], primaryRole: string) {
  const nextRoles = limitRuntimeRoles([
    primaryRole,
    ...roles.filter((role) => role !== primaryRole),
  ])
  if (selectedSupportRoleCount(nextRoles, primaryRole) > 0) return nextRoles
  return limitRuntimeRoles([
    ...nextRoles,
    manualSupportFallbackRoles(primaryRole)[0] ?? "ops",
  ])
}

export function limitRuntimeRoles(roles: string[]) {
  return uniqueRuntimeRoles(roles).slice(0, MAX_SELECTED_RUNTIME_ROLES)
}

export function selectedSupportRoleCount(
  roles: string[],
  primaryRole: string | undefined
) {
  return roles.filter((role) => role !== primaryRole).length
}

export function manualSupportFallbackRoles(primaryRole: string) {
  return ["ops", "data", "creative", "service", "engineering"].filter(
    (role) => role !== primaryRole
  )
}
