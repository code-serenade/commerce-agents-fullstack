import type { ReactNode } from "react"
import {
  Menubar as CoreMenubar,
  MenubarContent as CoreMenubarContent,
  MenubarItem as CoreMenubarItem,
  MenubarMenu as CoreMenubarMenu,
  MenubarTrigger as CoreMenubarTrigger,
} from "@workspace/ui-core/components/menubar"

export type MenubarItemVariant = "default" | "destructive"

export interface MenubarEntry {
  label: ReactNode
  disabled?: boolean
  variant?: MenubarItemVariant
  onSelect?: () => void
}

export interface MenubarMenuData {
  label: ReactNode
  items: MenubarEntry[]
}

export interface MenubarProps {
  menus: MenubarMenuData[]
}

export function Menubar({ menus }: MenubarProps) {
  return (
    <CoreMenubar>
      {menus.map((menu, index) => (
        <CoreMenubarMenu key={index}>
          <CoreMenubarTrigger>{menu.label}</CoreMenubarTrigger>
          <CoreMenubarContent>
            {menu.items.map((item, itemIndex) => (
              <CoreMenubarItem
                key={itemIndex}
                disabled={item.disabled}
                variant={item.variant}
                onSelect={item.onSelect}
              >
                {item.label}
              </CoreMenubarItem>
            ))}
          </CoreMenubarContent>
        </CoreMenubarMenu>
      ))}
    </CoreMenubar>
  )
}
