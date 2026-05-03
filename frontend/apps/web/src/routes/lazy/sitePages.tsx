import { lazy } from "react"

export const HomePage = lazy(() => import("@/pages/site/home"))

export const ProfilePage = lazy(() =>
  import("@/pages/site/shared-pages").then((module) => ({
    default: module.ProfilePage,
  }))
)
