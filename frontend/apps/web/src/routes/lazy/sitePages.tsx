import { lazy } from "react"

export const AgentChatPage = lazy(() => import("@/pages/site/agent-chat"))
export const AgentOutputPage = lazy(() => import("@/pages/site/agent-output"))
export const ApprovalsPage = lazy(() => import("@/pages/site/commerce/approvals"))
export const CampaignsPage = lazy(() => import("@/pages/site/commerce/campaigns"))
export const CommerceOverviewPage = lazy(
  () => import("@/pages/site/commerce/overview")
)
export const OperationsPage = lazy(() => import("@/pages/site/commerce/operations"))
export const PackagesPage = lazy(() => import("@/pages/site/commerce/packages"))
export const PlatformConnectPage = lazy(
  () => import("@/pages/site/commerce/platform-connect")
)
export const ProductsPage = lazy(() => import("@/pages/site/commerce/products"))
export const WorkspacesPage = lazy(() => import("@/pages/site/commerce/workspaces"))
export const HomePage = lazy(() => import("@/pages/site/home"))
export const ProfilePage = lazy(() =>
  import("@/pages/site/shared-pages").then((module) => ({
    default: module.ProfilePage,
  }))
)
