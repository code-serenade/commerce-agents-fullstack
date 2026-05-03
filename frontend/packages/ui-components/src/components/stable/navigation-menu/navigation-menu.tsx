import type { ReactNode } from "react"
import {
  NavigationMenu as CoreNavigationMenu,
  NavigationMenuContent as CoreNavigationMenuContent,
  NavigationMenuItem as CoreNavigationMenuItem,
  NavigationMenuLink as CoreNavigationMenuLink,
  NavigationMenuList as CoreNavigationMenuList,
  NavigationMenuTrigger as CoreNavigationMenuTrigger,
} from "@workspace/ui-core/components/navigation-menu"

export interface NavigationMenuSimpleLink {
  type: "link"
  label: ReactNode
  href: string
}

export interface NavigationMenuGroupLink {
  label: ReactNode
  href: string
}

export interface NavigationMenuGroup {
  type: "group"
  label: ReactNode
  links: NavigationMenuGroupLink[]
}

export type NavigationMenuItem = NavigationMenuSimpleLink | NavigationMenuGroup

export interface NavigationMenuProps {
  items: NavigationMenuItem[]
}

export function NavigationMenu({ items }: NavigationMenuProps) {
  return (
    <CoreNavigationMenu>
      <CoreNavigationMenuList>
        {items.map((item, index) => (
          <CoreNavigationMenuItem key={index}>
            {item.type === "group" ? (
              <>
                <CoreNavigationMenuTrigger>{item.label}</CoreNavigationMenuTrigger>
                <CoreNavigationMenuContent>
                  <ul className="grid gap-1 p-2 md:w-[260px]">
                    {item.links.map((link) => (
                      <li key={link.href}>
                        <CoreNavigationMenuLink href={link.href}>
                          {link.label}
                        </CoreNavigationMenuLink>
                      </li>
                    ))}
                  </ul>
                </CoreNavigationMenuContent>
              </>
            ) : (
              <CoreNavigationMenuLink href={item.href}>
                {item.label}
              </CoreNavigationMenuLink>
            )}
          </CoreNavigationMenuItem>
        ))}
      </CoreNavigationMenuList>
    </CoreNavigationMenu>
  )
}
