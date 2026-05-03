import { cva } from "class-variance-authority"

export const sidebarWrapperClassName =
  "group/sidebar-wrapper flex min-h-svh w-full has-data-[variant=inset]:bg-sidebar"

export const sidebarStaticClassName =
  "flex h-full w-(--sidebar-width) flex-col bg-[var(--surface)] text-sidebar-foreground"

export const sidebarGapVariants = cva(
  "relative w-(--sidebar-width) bg-transparent transition-[width] duration-200 ease-linear group-data-[collapsible=offcanvas]:w-0 group-data-[side=right]:rotate-180",
  {
    variants: {
      variant: {
        sidebar: "group-data-[collapsible=icon]:w-(--sidebar-width-icon)",
        floating:
          "group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4)))]",
        inset:
          "group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4)))]",
      },
    },
  }
)

export const sidebarContainerVariants = cva(
  "fixed inset-y-0 z-10 hidden h-svh w-(--sidebar-width) transition-[left,right,width] duration-200 ease-linear data-[side=left]:left-0 data-[side=left]:group-data-[collapsible=offcanvas]:left-[calc(var(--sidebar-width)*-1)] data-[side=right]:right-0 data-[side=right]:group-data-[collapsible=offcanvas]:right-[calc(var(--sidebar-width)*-1)] md:flex",
  {
    variants: {
      variant: {
        sidebar:
          "group-data-[collapsible=icon]:w-(--sidebar-width-icon) group-data-[side=left]:border-r group-data-[side=right]:border-l",
        floating:
          "p-2 group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4))+2px)]",
        inset:
          "p-2 group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4))+2px)]",
      },
    },
  }
)

export const sidebarTriggerSrOnlyClassName = "sr-only"

export const sidebarRailClassName =
  "absolute inset-y-0 z-20 hidden w-4 transition-all ease-linear group-data-[side=left]:-right-4 group-data-[side=right]:left-0 after:absolute after:inset-y-0 after:start-1/2 after:w-[2px] hover:after:bg-sidebar-border sm:flex ltr:-translate-x-1/2 rtl:-translate-x-1/2 in-data-[side=left]:cursor-w-resize in-data-[side=right]:cursor-e-resize [[data-side=left][data-state=collapsed]_&]:cursor-e-resize [[data-side=right][data-state=collapsed]_&]:cursor-w-resize group-data-[collapsible=offcanvas]:translate-x-0 group-data-[collapsible=offcanvas]:after:left-full hover:group-data-[collapsible=offcanvas]:bg-sidebar [[data-side=left][data-collapsible=offcanvas]_&]:-right-2 [[data-side=right][data-collapsible=offcanvas]_&]:-left-2"

export const sidebarInsetClassName =
  "relative flex min-h-0 min-w-0 w-full flex-1 flex-col bg-background md:peer-data-[variant=inset]:m-2 md:peer-data-[variant=inset]:ml-0 md:peer-data-[variant=inset]:rounded-xl md:peer-data-[variant=inset]:shadow-sm md:peer-data-[variant=inset]:peer-data-[state=collapsed]:ml-2"

export const sidebarInputClassName = "h-8 w-full bg-background shadow-none"

export const sidebarHeaderClassName = "flex flex-col gap-2 p-2"

export const sidebarFooterClassName = "flex flex-col gap-2 p-2"

export const sidebarSeparatorClassName =
  "mx-2 bg-sidebar-border data-horizontal:w-auto"

export const sidebarContentClassName =
  "no-scrollbar flex min-h-0 flex-1 flex-col gap-0 overflow-auto group-data-[collapsible=icon]:overflow-hidden"

export const sidebarGroupClassName = "relative flex w-full min-w-0 flex-col p-2"

export const sidebarGroupLabelClassName =
  "flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium text-sidebar-foreground/70 ring-sidebar-ring outline-hidden transition-[margin,opacity] duration-200 ease-linear group-data-[collapsible=icon]:-mt-8 group-data-[collapsible=icon]:opacity-0 focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0"

export const sidebarGroupActionClassName =
  "absolute top-3.5 right-3 flex aspect-square w-5 items-center justify-center rounded-md p-0 text-sidebar-foreground ring-sidebar-ring outline-hidden transition-transform group-data-[collapsible=icon]:hidden after:absolute after:-inset-2 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 md:after:hidden [&>svg]:size-4 [&>svg]:shrink-0"

export const sidebarGroupContentClassName = "w-full text-sm"

export const sidebarMenuClassName = "flex w-full min-w-0 flex-col gap-0"

export const sidebarMenuItemClassName = "group/menu-item relative"

export const sidebarMenuButtonVariants = cva(
  "peer/menu-button group/menu-button flex w-full items-center gap-2 overflow-hidden rounded-md p-2 text-left text-sm ring-sidebar-ring outline-hidden transition-[width,height,padding] group-has-data-[sidebar=menu-action]/menu-item:pr-8 group-data-[collapsible=icon]:w-8! group-data-[collapsible=icon]:gap-0 group-data-[collapsible=icon]:px-2! group-data-[collapsible=icon]:py-2! hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 group-data-[collapsible=icon]:data-[active-group=true]:bg-sidebar-accent group-data-[collapsible=icon]:data-[active-group=true]:text-sidebar-primary data-[active-group=true]:data-[disclosure-open=false]:bg-sidebar-accent data-[active-group=true]:data-[disclosure-open=false]:text-sidebar-primary data-open:hover:bg-sidebar-accent data-open:hover:text-sidebar-accent-foreground data-active:bg-sidebar-accent data-active:font-medium data-active:text-sidebar-accent-foreground [&_svg]:size-4 [&_svg]:shrink-0 [&>span:last-child]:truncate",
  {
    variants: {
      variant: {
        default: "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
        primary:
          "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground data-active:text-sidebar-primary",
        outline:
          "bg-background shadow-[0_0_0_1px_hsl(var(--sidebar-border))] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground hover:shadow-[0_0_0_1px_hsl(var(--sidebar-accent))]",
      },
      size: {
        default: "h-8 text-sm",
        sm: "h-7 text-xs",
        lg: "h-12 text-sm group-data-[collapsible=icon]:p-0!",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

export const sidebarDisclosureIconVariants = cva(
  "ml-auto shrink-0 transition-transform duration-200 ease-out group-data-[collapsible=icon]:hidden",
  {
    variants: {
      open: {
        false: "",
        true: "rotate-90",
      },
    },
  }
)

export const sidebarMenuActionVariants = cva(
  "absolute top-1.5 right-1 flex aspect-square w-5 items-center justify-center rounded-md p-0 text-sidebar-foreground ring-sidebar-ring outline-hidden transition-transform group-data-[collapsible=icon]:hidden peer-hover/menu-button:text-sidebar-accent-foreground peer-data-[size=default]/menu-button:top-1.5 peer-data-[size=lg]/menu-button:top-2.5 peer-data-[size=sm]/menu-button:top-1 after:absolute after:-inset-2 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 md:after:hidden [&>svg]:size-4 [&>svg]:shrink-0",
  {
    variants: {
      showOnHover: {
        false: "",
        true: "group-focus-within/menu-item:opacity-100 group-hover/menu-item:opacity-100 peer-data-active/menu-button:text-sidebar-accent-foreground aria-expanded:opacity-100 md:opacity-0",
      },
    },
  }
)

export const sidebarMenuBadgeClassName =
  "pointer-events-none absolute right-1 flex h-5 min-w-5 items-center justify-center rounded-md px-1 text-xs font-medium text-sidebar-foreground tabular-nums select-none group-data-[collapsible=icon]:hidden peer-hover/menu-button:text-sidebar-accent-foreground peer-data-[size=default]/menu-button:top-1.5 peer-data-[size=lg]/menu-button:top-2.5 peer-data-[size=sm]/menu-button:top-1 peer-data-active/menu-button:text-sidebar-accent-foreground"

export const sidebarMenuSkeletonClassName =
  "flex h-8 items-center gap-2 rounded-md px-2"

export const sidebarMenuSubClassName =
  "mx-3.5 flex min-w-0 translate-x-px flex-col gap-1 border-l border-sidebar-border px-2.5 py-0.5 group-data-[collapsible=icon]:hidden"

export const sidebarMenuSubItemClassName = "group/menu-sub-item relative"

export const sidebarMenuSubButtonVariants = cva(
  "flex h-7 min-w-0 -translate-x-px items-center gap-2 overflow-hidden rounded-md px-2 text-sidebar-foreground ring-sidebar-ring outline-hidden group-data-[collapsible=icon]:hidden hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 data-active:bg-sidebar-accent data-active:font-medium data-active:text-sidebar-primary data-active:hover:bg-sidebar-accent data-active:hover:text-sidebar-primary [&>span:last-child]:truncate [&>svg]:size-4 [&>svg]:shrink-0 [&>svg]:text-sidebar-accent-foreground",
  {
    variants: {
      size: {
        md: "text-sm",
        sm: "text-xs",
      },
    },
    defaultVariants: {
      size: "md",
    },
  }
)

export const sidebarClassNames = {
  slot1:
    "w-(--sidebar-width) bg-[var(--surface)] p-0 text-sidebar-foreground [&>button]:hidden",
  slot2: "sr-only",
  slot3: "flex h-full w-full flex-col",
  slot4: "group peer hidden text-sidebar-foreground md:block",
  slot5:
    "flex size-full flex-col border border-[var(--border)] bg-[var(--surface)] group-data-[variant=floating]:rounded-lg group-data-[variant=floating]:shadow-sm group-data-[variant=floating]:ring-1 group-data-[variant=floating]:ring-sidebar-border",
  slot6: "size-4 rounded-md",
  slot7: "h-4 max-w-(--skeleton-width) flex-1",
} as const
