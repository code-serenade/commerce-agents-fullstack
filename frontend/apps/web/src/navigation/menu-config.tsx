import type { ReactNode } from "react"
import { House } from "lucide-react"

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
        labelKey: "web.shell.navigation.home",
        path: "/home",
        icon: <House />,
        matcher: (pathname) => pathname === "/" || pathname === "/home",
      },
    ],
  },
]
