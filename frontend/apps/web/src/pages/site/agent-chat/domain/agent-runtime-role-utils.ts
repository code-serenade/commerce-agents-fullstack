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
