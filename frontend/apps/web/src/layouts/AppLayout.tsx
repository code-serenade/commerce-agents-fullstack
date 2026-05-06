import { Outlet } from "react-router"
import { AgentChatShell } from "@/shell"

export default function AppLayout() {
  return (
    <AgentChatShell>
      <Outlet />
    </AgentChatShell>
  )
}
