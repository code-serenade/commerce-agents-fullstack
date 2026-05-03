import * as React from "react"
import type { VariantProps } from "class-variance-authority"

import type { BaseMode } from "../../lib/component-mode"
import type { ButtonProps } from "../button"
import type { TooltipContentProps } from "../tooltip"
import { sidebarMenuButtonVariants } from "./sidebar.styles"

export type SidebarClassNameMode = "merge" | "replace"

export type SidebarClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type SidebarSide = "left" | "right"
export type SidebarVariant = "sidebar" | "floating" | "inset"
export type SidebarCollapsible = "offcanvas" | "icon" | "none"

export type SidebarProviderProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  defaultOpen?: boolean
  open?: boolean
  onOpenChange?: (open: boolean) => void
}

export type SidebarProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  side?: SidebarSide
  variant?: SidebarVariant
  collapsible?: SidebarCollapsible
}

export type SidebarTriggerProps = ButtonProps & {
  mode?: BaseMode
  iconClassName?: string
  textClassName?: string
}

export type SidebarRailProps = React.ComponentProps<"button"> & {
  mode?: BaseMode
}

export type SidebarInsetProps = React.ComponentProps<"main"> & {
  mode?: BaseMode
}

export type SidebarInputProps = React.ComponentProps<"input"> & {
  mode?: BaseMode
}

export type SidebarHeaderProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
}

export type SidebarFooterProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
}

export type SidebarSeparatorProps = React.ComponentProps<"hr"> & {
  mode?: BaseMode
}

export type SidebarContentProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
}

export type SidebarGroupProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
}

export type SidebarGroupLabelProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  asChild?: boolean
}

export type SidebarGroupActionProps = React.ComponentProps<"button"> & {
  mode?: BaseMode
  asChild?: boolean
}

export type SidebarGroupContentProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
}

export type SidebarMenuProps = React.ComponentProps<"ul"> & {
  mode?: BaseMode
}

export type SidebarMenuItemProps = React.ComponentProps<"li"> & {
  mode?: BaseMode
}

export type SidebarMenuButtonProps = React.ComponentProps<"button"> & {
  mode?: BaseMode
  asChild?: boolean
  isActive?: boolean
  isActiveGroup?: boolean
  disclosure?: boolean
  disclosureOpen?: boolean
  tooltip?: string | Omit<TooltipContentProps, "mode">
} & VariantProps<typeof sidebarMenuButtonVariants>

export type SidebarMenuActionProps = React.ComponentProps<"button"> & {
  mode?: BaseMode
  asChild?: boolean
  showOnHover?: boolean
}

export type SidebarMenuBadgeProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
}

export type SidebarMenuSkeletonProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  showIcon?: boolean
}

export type SidebarMenuSubProps = React.ComponentProps<"ul"> & {
  mode?: BaseMode
}

export type SidebarMenuSubItemProps = React.ComponentProps<"li"> & {
  mode?: BaseMode
}

export type SidebarMenuSubButtonProps = React.ComponentProps<"a"> & {
  mode?: BaseMode
  asChild?: boolean
  size?: "sm" | "md"
  isActive?: boolean
}
