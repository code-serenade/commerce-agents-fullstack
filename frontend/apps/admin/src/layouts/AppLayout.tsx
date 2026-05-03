import { Outlet } from "react-router"
import { AdminAppShell } from "@/shell"

export default function AppLayout() {
  return <AdminAppShell><Outlet /></AdminAppShell>
}
