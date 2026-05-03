import { useSelector } from "react-redux"
import { Navigate, Outlet, useLocation } from "react-router"
import type { RootState } from "@/store"

export function RequireAuth() {
  const isLogin = useSelector((state: RootState) => state.auth.isLogin)
  const location = useLocation()

  if (!isLogin) {
    return <Navigate to="/login" replace state={{ from: location }} />
  }

  return <Outlet />
}
