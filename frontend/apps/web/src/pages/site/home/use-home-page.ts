import { useSelector } from "react-redux"
import type { RootState } from "@/store"

export function useHomePage() {
  const user = useSelector((state: RootState) => state.auth.user)
  const displayName = user?.name || user?.display_id || user?.email || ""
  const displayId = user?.display_id || user?.id || ""

  return {
    user,
    displayName,
    displayId,
    avatarFallback: displayName.charAt(0).toUpperCase() || "U",
  }
}
