import { Outlet, useLocation } from "react-router"
import { AgentChatShell, WebAppShell } from "@/shell"
import { isAgentShellRoute } from "./app-layout-routes"

export default function AppLayout() {
  const location = useLocation()
  const isAgentChatRoute = isAgentShellRoute(location.pathname)

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
