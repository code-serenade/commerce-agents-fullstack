import { Outlet, useLocation } from "react-router"
import { AgentChatShell, WebAppShell } from "@/shell"

export default function AppLayout() {
  const location = useLocation()
  const isAgentChatRoute =
    location.pathname === "/" ||
    location.pathname === "/commerce" ||
    location.pathname === "/agent" ||
    location.pathname === "/agent-capabilities" ||
    location.pathname === "/overview" ||
    location.pathname === "/products" ||
    location.pathname === "/campaigns" ||
    location.pathname === "/workspaces" ||
    location.pathname === "/platform-connect" ||
    location.pathname === "/approvals" ||
    location.pathname === "/operations" ||
    location.pathname === "/agent-output"

  if (isAgentChatRoute) {
    return (
      <AgentChatShell>
        <Outlet />
      </AgentChatShell>
    )
  }

  return (
    <WebAppShell>
      <Outlet />
    </WebAppShell>
  )
}
