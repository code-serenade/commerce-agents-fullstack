const AGENT_SHELL_ROUTE_PATHS = new Set([
  "/",
  "/commerce",
  "/agent",
  "/agent-capabilities",
  "/overview",
  "/materials",
  "/products",
  "/campaigns",
  "/workspaces",
  "/platform-connect",
  "/approvals",
  "/operations",
  "/agent-output",
])

export function isAgentShellRoute(pathname: string) {
  return AGENT_SHELL_ROUTE_PATHS.has(pathname)
}
