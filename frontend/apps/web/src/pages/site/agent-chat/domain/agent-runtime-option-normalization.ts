import type { AgentRuntimeOptions } from "./agent-runtime-types"

export function normalizeRuntimeOptions(
  options: AgentRuntimeOptions | null
): AgentRuntimeOptions | null {
  if (!options) return null
  const seenRuntimeRoles = new Set<string>()
  return {
    ...options,
    roles: options.roles
      .map((role) => ({
        ...role,
        runtimeRole: normalizeRuntimeRoleOption(role.runtimeRole),
      }))
      .filter((role) => {
        if (seenRuntimeRoles.has(role.runtimeRole)) return false
        seenRuntimeRoles.add(role.runtimeRole)
        return true
      }),
  }
}

function normalizeRuntimeRoleOption(runtimeRole: string) {
  const normalized = runtimeRole.toLowerCase()
  if (normalized.includes("douyin") || normalized.includes("qianchuan")) {
    return "ops"
  }
  return runtimeRole
}
