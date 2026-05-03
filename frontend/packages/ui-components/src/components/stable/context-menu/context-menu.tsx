import type { ReactNode } from "react"
import {
  ContextMenu as CoreContextMenu,
  ContextMenuContent as CoreContextMenuContent,
  ContextMenuItem as CoreContextMenuItem,
  ContextMenuTrigger as CoreContextMenuTrigger,
} from "@workspace/ui-core/components/context-menu"

export type ContextMenuItemVariant = "default" | "destructive"

export interface ContextMenuEntry {
  label: ReactNode
  disabled?: boolean
  variant?: ContextMenuItemVariant
  onSelect?: () => void
}

export interface ContextMenuProps {
  trigger: ReactNode
  items: ContextMenuEntry[]
}

export function ContextMenu({ trigger, items }: ContextMenuProps) {
  return (
    <CoreContextMenu>
      <CoreContextMenuTrigger>
        <div className="rounded-lg border px-4 py-8 text-center text-sm text-muted-foreground">
          {trigger}
        </div>
      </CoreContextMenuTrigger>
      <CoreContextMenuContent>
        {items.map((item, index) => (
          <CoreContextMenuItem
            key={index}
            disabled={item.disabled}
            variant={item.variant}
            onSelect={item.onSelect}
          >
            {item.label}
          </CoreContextMenuItem>
        ))}
      </CoreContextMenuContent>
    </CoreContextMenu>
  )
}
