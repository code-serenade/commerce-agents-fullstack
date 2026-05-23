export type AgentRoutingMode = "commander" | "fast"

export function normalizeRoutingMode(
  value: string | null | undefined
): AgentRoutingMode {
  return value === "fast" ? "fast" : "commander"
}
