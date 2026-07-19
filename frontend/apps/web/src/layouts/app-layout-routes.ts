const AGENT_SHELL_ROUTE_PATHS = new Set([
  "/",
  "/home",
  "/commerce",
  "/agent",
  "/agent-capabilities",
  "/question-ideas",
  "/overview",
  "/materials",
  "/products",
  "/campaigns",
  "/workspaces",
  "/platform-connect",
  "/approvals",
  "/operations",
  "/agent-output",
  "/profile",
  "/team-school",
])

export function isAgentShellRoute(pathname: string) {
  const normalizedPathname = normalizePathname(pathname)

  return Array.from(AGENT_SHELL_ROUTE_PATHS).some(
    (path) =>
      normalizedPathname === path ||
      (path !== "/" && normalizedPathname.startsWith(`${path}/`))
  )
}

function normalizePathname(pathname: string) {
  if (pathname === "/") return pathname
  return pathname.replace(/\/+$/, "")
}
