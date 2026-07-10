import { Outlet, useLocation } from "react-router"
import { AgentChatShell, WebAppShell } from "@/shell"
import { ProductAnalyticsTracker } from "@/analytics/ProductAnalyticsTracker"
import { isAgentShellRoute } from "./app-layout-routes"
import { AuthDialogProvider } from "@/components/auth/AuthDialogProvider"

export default function AppLayout() {
  const location = useLocation()
  const isAgentChatRoute = isAgentShellRoute(location.pathname)

  return (
    <AuthDialogProvider>
      {isAgentChatRoute ? (
        <AgentChatShell>
          <ProductAnalyticsTracker />
          <Outlet />
        </AgentChatShell>
      ) : (
        <WebAppShell>
          <ProductAnalyticsTracker />
          <Outlet />
        </WebAppShell>
      )}
    </AuthDialogProvider>
  )
}
