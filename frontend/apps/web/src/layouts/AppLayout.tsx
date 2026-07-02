import { Outlet, useLocation } from "react-router"
import { AgentChatShell, WebAppShell } from "@/shell"
import { ProductAnalyticsTracker } from "@/analytics/ProductAnalyticsTracker"
import { isAgentShellRoute } from "./app-layout-routes"

export default function AppLayout() {
  const location = useLocation()
  const isAgentChatRoute = isAgentShellRoute(location.pathname)

  if (isAgentChatRoute) {
    return (
      <AgentChatShell>
        <ProductAnalyticsTracker />
        <Outlet />
      </AgentChatShell>
    )
  }

  return (
    <WebAppShell>
      <ProductAnalyticsTracker />
      <Outlet />
    </WebAppShell>
  )
}
