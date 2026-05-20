import type { ReactNode } from "react"
import {
  BellRing,
  Bot,
  Boxes,
  ClipboardCheck,
  FileText,
  LayoutDashboard,
  Megaphone,
  PlugZap,
  ShoppingBag,
} from "lucide-react"

export interface WebNavigationItemConfig {
  id: string
  labelKey: string
  path: string
  icon?: ReactNode
  matcher?: (pathname: string) => boolean
}

export interface WebNavigationSectionConfig {
  id: string
  labelKey: string
  items: WebNavigationItemConfig[]
}

export const webNavigationSections: WebNavigationSectionConfig[] = [
  {
    id: "workspace",
    labelKey: "web.shell.navigation.workspace",
    items: [
      {
        id: "home",
        labelKey: "web.shell.navigation.agent",
        path: "/commerce",
        icon: <Bot />,
        matcher: (pathname) => pathname === "/" || pathname === "/commerce",
      },
      {
        id: "overview",
        labelKey: "web.shell.navigation.home",
        path: "/overview",
        icon: <LayoutDashboard />,
      },
      {
        id: "products",
        labelKey: "web.shell.navigation.products",
        path: "/products",
        icon: <ShoppingBag />,
      },
      {
        id: "campaigns",
        labelKey: "web.shell.navigation.campaigns",
        path: "/campaigns",
        icon: <Megaphone />,
      },
      {
        id: "workspaces",
        labelKey: "web.shell.navigation.workspaces",
        path: "/workspaces",
        icon: <Boxes />,
      },
      {
        id: "platform-connect",
        labelKey: "web.shell.navigation.platformConnect",
        path: "/platform-connect",
        icon: <PlugZap />,
      },
    ],
  },
  {
    id: "governance",
    labelKey: "web.shell.navigation.governance",
    items: [
      {
        id: "approvals",
        labelKey: "web.shell.navigation.approvals",
        path: "/approvals",
        icon: <ClipboardCheck />,
      },
      {
        id: "operations",
        labelKey: "web.shell.navigation.operations",
        path: "/operations",
        icon: <BellRing />,
      },
    ],
  },
  {
    id: "agent",
    labelKey: "web.shell.navigation.agentSection",
    items: [
      {
        id: "agent-output",
        labelKey: "web.shell.navigation.agentOutput",
        path: "/agent-output",
        icon: <FileText />,
      },
    ],
  },
]
