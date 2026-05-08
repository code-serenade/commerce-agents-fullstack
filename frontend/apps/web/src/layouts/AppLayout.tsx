import { Outlet, useLocation } from "react-router"
import { AgentChatShell, WebAppShell } from "@/shell"

export default function AppLayout() {
  const location = useLocation()
  const isAgentChatRoute =
    location.pathname === "/" ||
    location.pathname === "/commerce" ||
    location.pathname === "/agent"

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
