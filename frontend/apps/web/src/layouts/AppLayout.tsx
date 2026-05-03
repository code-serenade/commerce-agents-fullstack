import { Outlet } from "react-router"
import { WebAppShell } from "@/shell"

export default function AppLayout() {
  return <WebAppShell><Outlet /></WebAppShell>
}
