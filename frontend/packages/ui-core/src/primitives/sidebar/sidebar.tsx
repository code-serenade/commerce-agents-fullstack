"use client"

import * as React from "react"
import { Slot } from "radix-ui"

import { useIsMobile } from "../../hooks/use-mobile"
import { DEFAULT_MODE } from "../../lib/component-mode"
import { ChevronRightIcon, PanelLeftIcon } from "../../lib/icon-slots"
import { cn } from "../../lib/utils"
import { Button } from "../button"
import { Input } from "../input"
import { Separator } from "../separator"
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "../sheet"
import { Skeleton } from "../skeleton"
import { Tooltip, TooltipContent, TooltipTrigger } from "../tooltip"
import {
  SidebarContext,
  type SidebarContextProps,
  useSidebar,
} from "./sidebar.context"
import {
  sidebarContainerVariants,
  sidebarClassNames,
  sidebarContentClassName,
  sidebarDisclosureIconVariants,
  sidebarFooterClassName,
  sidebarGapVariants,
  sidebarGroupActionClassName,
  sidebarGroupClassName,
  sidebarGroupContentClassName,
  sidebarGroupLabelClassName,
  sidebarHeaderClassName,
  sidebarInputClassName,
  sidebarInsetClassName,
  sidebarMenuActionVariants,
  sidebarMenuBadgeClassName,
  sidebarMenuButtonVariants,
  sidebarMenuClassName,
  sidebarMenuItemClassName,
  sidebarMenuSkeletonClassName,
  sidebarMenuSubButtonVariants,
  sidebarMenuSubClassName,
  sidebarMenuSubItemClassName,
  sidebarRailClassName,
  sidebarSeparatorClassName,
  sidebarStaticClassName,
  sidebarTriggerSrOnlyClassName,
  sidebarWrapperClassName,
} from "./sidebar.styles"
import type {
  SidebarContentProps,
  SidebarFooterProps,
  SidebarGroupActionProps,
  SidebarGroupContentProps,
  SidebarGroupLabelProps,
  SidebarGroupProps,
  SidebarHeaderProps,
  SidebarInputProps,
  SidebarInsetProps,
  SidebarMenuActionProps,
  SidebarMenuBadgeProps,
  SidebarMenuButtonProps,
  SidebarMenuItemProps,
  SidebarMenuProps,
  SidebarMenuSkeletonProps,
  SidebarMenuSubButtonProps,
  SidebarMenuSubItemProps,
  SidebarMenuSubProps,
  SidebarProps,
  SidebarProviderProps,
  SidebarRailProps,
  SidebarSeparatorProps,
  SidebarTriggerProps,
} from "./sidebar.types"

const SIDEBAR_COOKIE_NAME = "sidebar_state"
const SIDEBAR_WIDTH_COOKIE_NAME = "sidebar_width"
const SIDEBAR_COOKIE_MAX_AGE = 60 * 60 * 24 * 7
const SIDEBAR_WIDTH = "16rem"
const SIDEBAR_WIDTH_MOBILE = "18rem"
const SIDEBAR_WIDTH_ICON = "3.5rem"
const SIDEBAR_WIDTH_MIN = "10rem"
const SIDEBAR_KEYBOARD_SHORTCUT = "b"

function parseRemSize(value: string) {
  return Number.parseFloat(value)
}

function clampValue(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max)
}

function getRootFontSize() {
  if (typeof window === "undefined") return 16

  return (
    Number.parseFloat(
      window.getComputedStyle(document.documentElement).fontSize
    ) || 16
  )
}

function SidebarProvider({
  mode = DEFAULT_MODE,
  defaultOpen = true,
  open: openProp,
  onOpenChange: setOpenProp,
  className,
  style,
  children,
  ...props
}: SidebarProviderProps) {
  const isMobile = useIsMobile()
  const [openMobile, setOpenMobile] = React.useState(false)
  const [sidebarWidth, setSidebarWidthState] = React.useState(() => {
    const rootFontSize = getRootFontSize()
    return parseRemSize(SIDEBAR_WIDTH) * rootFontSize
  })

  const [_open, _setOpen] = React.useState(defaultOpen)
  const open = openProp ?? _open
  const sidebarWidthMin = React.useMemo(
    () => parseRemSize(SIDEBAR_WIDTH_MIN) * getRootFontSize(),
    []
  )
  const sidebarWidthMax = React.useMemo(
    () => parseRemSize(SIDEBAR_WIDTH) * getRootFontSize(),
    []
  )
  const setSidebarWidth = React.useCallback(
    (width: number) => {
      const nextWidth = clampValue(width, sidebarWidthMin, sidebarWidthMax)
      setSidebarWidthState(nextWidth)
      document.cookie = `${SIDEBAR_WIDTH_COOKIE_NAME}=${nextWidth}; path=/; max-age=${SIDEBAR_COOKIE_MAX_AGE}`
    },
    [sidebarWidthMax, sidebarWidthMin]
  )

  React.useEffect(() => {
    const savedWidth = document.cookie
      .split("; ")
      .find((entry) => entry.startsWith(`${SIDEBAR_WIDTH_COOKIE_NAME}=`))
      ?.split("=")[1]

    if (!savedWidth) return

    const parsedWidth = Number.parseFloat(savedWidth)
    if (Number.isNaN(parsedWidth)) return

    setSidebarWidth(parsedWidth)
  }, [setSidebarWidth])

  const setOpen = React.useCallback(
    (value: boolean | ((value: boolean) => boolean)) => {
      const openState = typeof value === "function" ? value(open) : value
      if (setOpenProp) {
        setOpenProp(openState)
      } else {
        _setOpen(openState)
      }

      document.cookie = `${SIDEBAR_COOKIE_NAME}=${openState}; path=/; max-age=${SIDEBAR_COOKIE_MAX_AGE}`
    },
    [setOpenProp, open]
  )

  const toggleSidebar = React.useCallback(() => {
    return isMobile
      ? setOpenMobile((nextOpen) => !nextOpen)
      : setOpen((nextOpen) => !nextOpen)
  }, [isMobile, setOpen, setOpenMobile])

  React.useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (
        event.key === SIDEBAR_KEYBOARD_SHORTCUT &&
        (event.metaKey || event.ctrlKey)
      ) {
        event.preventDefault()
        toggleSidebar()
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [toggleSidebar])

  const state = open ? "expanded" : "collapsed"

  const contextValue = React.useMemo<SidebarContextProps>(
    () => ({
      state,
      open,
      setOpen,
      isMobile,
      openMobile,
      setOpenMobile,
      toggleSidebar,
      sidebarWidth,
      sidebarWidthMin,
      sidebarWidthMax,
      setSidebarWidth,
    }),
    [
      state,
      open,
      setOpen,
      isMobile,
      openMobile,
      setOpenMobile,
      toggleSidebar,
      sidebarWidth,
      sidebarWidthMin,
      sidebarWidthMax,
      setSidebarWidth,
    ]
  )

  if (mode === "primitive") {
    return (
      <SidebarContext.Provider value={contextValue}>
        <div className={className} style={style} {...props}>
          {children}
        </div>
      </SidebarContext.Provider>
    )
  }

  return (
    <SidebarContext.Provider value={contextValue}>
      <div
        data-slot="sidebar-wrapper"
        style={
          {
            "--sidebar-width": `${sidebarWidth}px`,
            "--sidebar-width-icon": SIDEBAR_WIDTH_ICON,
            ...style,
          } as React.CSSProperties
        }
        className={cn(sidebarWrapperClassName, className)}
        {...props}
      >
        {children}
      </div>
    </SidebarContext.Provider>
  )
}

function Sidebar({
  mode = DEFAULT_MODE,
  side = "left",
  variant = "sidebar",
  collapsible = "offcanvas",
  className,
  children,
  dir,
  ...props
}: SidebarProps) {
  const { isMobile, state, openMobile, setOpenMobile } = useSidebar()

  if (collapsible === "none") {
    if (mode === "primitive") {
      return (
        <div className={className} {...props}>
          {children}
        </div>
      )
    }

    return (
      <div
        data-slot="sidebar"
        className={cn(sidebarStaticClassName, className)}
        {...props}
      >
        {children}
      </div>
    )
  }

  if (isMobile) {
    if (mode === "primitive") {
      return (
        <Sheet mode={mode} open={openMobile} onOpenChange={setOpenMobile}>
          <SheetContent mode={mode} dir={dir} side={side}>
            {children}
          </SheetContent>
        </Sheet>
      )
    }

    return (
      <Sheet
        mode={mode}
        open={openMobile}
        onOpenChange={setOpenMobile}
        {...props}
      >
        <SheetContent
          mode={mode}
          dir={dir}
          data-sidebar="sidebar"
          data-slot="sidebar"
          data-mobile="true"
          className={sidebarClassNames.slot1}
          style={
            {
              "--sidebar-width": SIDEBAR_WIDTH_MOBILE,
            } as React.CSSProperties
          }
          side={side}
        >
          <SheetHeader mode={mode} className={sidebarClassNames.slot2}>
            <SheetTitle mode={mode}>Sidebar</SheetTitle>
            <SheetDescription mode={mode}>
              Displays the mobile sidebar.
            </SheetDescription>
          </SheetHeader>
          <div className={sidebarClassNames.slot3}>{children}</div>
        </SheetContent>
      </Sheet>
    )
  }

  if (mode === "primitive") {
    return (
      <div className={className} {...props}>
        {children}
      </div>
    )
  }

  return (
    <div
      className={sidebarClassNames.slot4}
      data-state={state}
      data-collapsible={state === "collapsed" ? collapsible : ""}
      data-variant={variant}
      data-side={side}
      data-slot="sidebar"
    >
      <div
        data-slot="sidebar-gap"
        className={cn(sidebarGapVariants({ variant }))}
      />
      <div
        data-slot="sidebar-container"
        data-side={side}
        className={cn(sidebarContainerVariants({ variant }), className)}
        {...props}
      >
        <div
          data-sidebar="sidebar"
          data-slot="sidebar-inner"
          className={sidebarClassNames.slot5}
        >
          {children}
        </div>
      </div>
    </div>
  )
}

function SidebarTrigger({
  mode = DEFAULT_MODE,
  iconClassName,
  textClassName,
  className,
  onClick,
  ...props
}: SidebarTriggerProps) {
  const { toggleSidebar } = useSidebar()

  if (mode === "primitive") {
    const {
      asChild: _asChild,
      variant: _variant,
      color: _color,
      size: _size,
      classNameMode: _classNameMode,
      classResolver: _classResolver,
      ...rest
    } = props

    return (
      <button
        className={className}
        onClick={(event) => {
          onClick?.(event)
          toggleSidebar()
        }}
        {...rest}
      >
        <PanelLeftIcon className={iconClassName} />
        <span className={textClassName}>Toggle Sidebar</span>
      </button>
    )
  }

  return (
    <Button
      data-sidebar="trigger"
      data-slot="sidebar-trigger"
      variant="ghost"
      size="icon-sm"
      className={cn(className)}
      onClick={(event) => {
        onClick?.(event)
        toggleSidebar()
      }}
      {...props}
    >
      <PanelLeftIcon className={iconClassName} />
      <span className={cn(sidebarTriggerSrOnlyClassName, textClassName)}>
        Toggle Sidebar
      </span>
    </Button>
  )
}

function SidebarRail({
  mode = DEFAULT_MODE,
  className,
  onClick,
  ...props
}: SidebarRailProps) {
  const {
    isMobile,
    open,
    setOpen,
    sidebarWidth,
    sidebarWidthMin,
    sidebarWidthMax,
    setSidebarWidth,
    toggleSidebar,
  } = useSidebar()
  const suppressClickRef = React.useRef(false)

  const handlePointerDown = React.useCallback(
    (event: React.PointerEvent<HTMLButtonElement>) => {
      if (mode === "primitive" || isMobile) return

      event.preventDefault()

      const side =
        event.currentTarget.closest<HTMLElement>("[data-side]")?.dataset
          .side === "right"
          ? "right"
          : "left"
      const startX = event.clientX
      const startWidth = open ? sidebarWidth : sidebarWidthMin
      suppressClickRef.current = false

      if (!open) {
        setOpen(true)
      }

      const handlePointerMove = (moveEvent: PointerEvent) => {
        const delta = moveEvent.clientX - startX
        const signedDelta = side === "right" ? -delta : delta
        const nextWidth = clampValue(
          startWidth + signedDelta,
          sidebarWidthMin,
          sidebarWidthMax
        )

        if (Math.abs(nextWidth - startWidth) > 2) {
          suppressClickRef.current = true
        }

        setSidebarWidth(nextWidth)
      }

      const handlePointerUp = () => {
        window.removeEventListener("pointermove", handlePointerMove)
        window.removeEventListener("pointerup", handlePointerUp)
      }

      window.addEventListener("pointermove", handlePointerMove)
      window.addEventListener("pointerup", handlePointerUp)
    },
    [
      isMobile,
      mode,
      open,
      setOpen,
      setSidebarWidth,
      sidebarWidth,
      sidebarWidthMax,
      sidebarWidthMin,
    ]
  )

  if (mode === "primitive") {
    return (
      <button
        aria-label="Toggle Sidebar"
        tabIndex={-1}
        onClick={(event) => {
          onClick?.(event)
          toggleSidebar()
        }}
        title="Toggle Sidebar"
        className={className}
        {...props}
      />
    )
  }

  return (
    <button
      data-sidebar="rail"
      data-slot="sidebar-rail"
      aria-label="Toggle Sidebar"
      tabIndex={-1}
      onClick={(event) => {
        if (suppressClickRef.current) {
          suppressClickRef.current = false
          return
        }
        onClick?.(event)
        toggleSidebar()
      }}
      onPointerDown={handlePointerDown}
      title="Toggle Sidebar"
      className={cn(sidebarRailClassName, className)}
      {...props}
    />
  )
}

function SidebarInset({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarInsetProps) {
  if (mode === "primitive") {
    return <main className={className} {...props} />
  }

  return (
    <main
      data-slot="sidebar-inset"
      className={cn(sidebarInsetClassName, className)}
      {...props}
    />
  )
}

function SidebarInput({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarInputProps) {
  if (mode === "primitive") {
    return <Input mode={mode} className={className} {...props} />
  }

  return (
    <Input
      mode={mode}
      data-slot="sidebar-input"
      data-sidebar="input"
      className={cn(sidebarInputClassName, className)}
      {...props}
    />
  )
}

function SidebarHeader({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarHeaderProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="sidebar-header"
      data-sidebar="header"
      className={cn(sidebarHeaderClassName, className)}
      {...props}
    />
  )
}

function SidebarFooter({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarFooterProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="sidebar-footer"
      data-sidebar="footer"
      className={cn(sidebarFooterClassName, className)}
      {...props}
    />
  )
}

function SidebarSeparator({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarSeparatorProps) {
  if (mode === "primitive") {
    return <Separator mode={mode} className={className} {...props} />
  }

  return (
    <Separator
      mode={mode}
      data-slot="sidebar-separator"
      data-sidebar="separator"
      className={cn(sidebarSeparatorClassName, className)}
      {...props}
    />
  )
}

function SidebarContent({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarContentProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="sidebar-content"
      data-sidebar="content"
      className={cn(sidebarContentClassName, className)}
      {...props}
    />
  )
}

function SidebarGroup({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarGroupProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="sidebar-group"
      data-sidebar="group"
      className={cn(sidebarGroupClassName, className)}
      {...props}
    />
  )
}

function SidebarGroupLabel({
  mode = DEFAULT_MODE,
  className,
  asChild = false,
  ...props
}: SidebarGroupLabelProps) {
  const Comp = asChild ? Slot.Root : "div"

  if (mode === "primitive") {
    return <Comp className={className} {...props} />
  }

  return (
    <Comp
      data-slot="sidebar-group-label"
      data-sidebar="group-label"
      className={cn(sidebarGroupLabelClassName, className)}
      {...props}
    />
  )
}

function SidebarGroupAction({
  mode = DEFAULT_MODE,
  className,
  asChild = false,
  ...props
}: SidebarGroupActionProps) {
  const Comp = asChild ? Slot.Root : "button"

  if (mode === "primitive") {
    return <Comp className={className} {...props} />
  }

  return (
    <Comp
      data-slot="sidebar-group-action"
      data-sidebar="group-action"
      className={cn(sidebarGroupActionClassName, className)}
      {...props}
    />
  )
}

function SidebarGroupContent({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarGroupContentProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="sidebar-group-content"
      data-sidebar="group-content"
      className={cn(sidebarGroupContentClassName, className)}
      {...props}
    />
  )
}

function SidebarMenu({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarMenuProps) {
  if (mode === "primitive") {
    return <ul className={className} {...props} />
  }

  return (
    <ul
      data-slot="sidebar-menu"
      data-sidebar="menu"
      className={cn(sidebarMenuClassName, className)}
      {...props}
    />
  )
}

function SidebarMenuItem({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarMenuItemProps) {
  if (mode === "primitive") {
    return <li className={className} {...props} />
  }

  return (
    <li
      data-slot="sidebar-menu-item"
      data-sidebar="menu-item"
      className={cn(sidebarMenuItemClassName, className)}
      {...props}
    />
  )
}

function SidebarMenuButton({
  mode = DEFAULT_MODE,
  asChild = false,
  isActive = false,
  isActiveGroup = false,
  disclosure = false,
  disclosureOpen = false,
  variant = "default",
  size = "default",
  tooltip,
  className,
  ...props
}: SidebarMenuButtonProps) {
  const Comp = asChild ? Slot.Root : "button"
  const { isMobile, state } = useSidebar()

  if (mode === "primitive") {
    const button = <Comp className={className} {...props} />

    if (!tooltip) {
      return button
    }

    const tooltipProps =
      typeof tooltip === "string" ? { children: tooltip } : tooltip

    return (
      <Tooltip mode={mode}>
        <TooltipTrigger mode={mode} asChild>
          {button}
        </TooltipTrigger>
        <TooltipContent
          mode={mode}
          side="right"
          align="center"
          hidden={state !== "collapsed" || isMobile}
          {...tooltipProps}
        />
      </Tooltip>
    )
  }

  const button = (
    <Comp
      data-slot="sidebar-menu-button"
      data-sidebar="menu-button"
      data-size={size}
      data-active={isActive}
      data-active-group={isActiveGroup}
      data-disclosure={disclosure}
      data-disclosure-open={disclosureOpen}
      aria-expanded={disclosure ? disclosureOpen : undefined}
      className={cn(sidebarMenuButtonVariants({ variant, size }), className)}
      {...props}
    >
      {props.children}
      {disclosure ? (
        <ChevronRightIcon
          aria-hidden="true"
          className={cn(
            sidebarDisclosureIconVariants({ open: disclosureOpen })
          )}
        />
      ) : null}
    </Comp>
  )

  if (!tooltip) {
    return button
  }

  const tooltipProps =
    typeof tooltip === "string" ? { children: tooltip } : tooltip

  return (
    <Tooltip mode={mode}>
      <TooltipTrigger mode={mode} asChild>
        {button}
      </TooltipTrigger>
      <TooltipContent
        mode={mode}
        side="right"
        align="center"
        hidden={state !== "collapsed" || isMobile}
        {...tooltipProps}
      />
    </Tooltip>
  )
}

function SidebarMenuAction({
  mode = DEFAULT_MODE,
  className,
  asChild = false,
  showOnHover = false,
  ...props
}: SidebarMenuActionProps) {
  const Comp = asChild ? Slot.Root : "button"

  if (mode === "primitive") {
    return <Comp className={className} {...props} />
  }

  return (
    <Comp
      data-slot="sidebar-menu-action"
      data-sidebar="menu-action"
      className={cn(sidebarMenuActionVariants({ showOnHover }), className)}
      {...props}
    />
  )
}

function SidebarMenuBadge({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarMenuBadgeProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="sidebar-menu-badge"
      data-sidebar="menu-badge"
      className={cn(sidebarMenuBadgeClassName, className)}
      {...props}
    />
  )
}

function SidebarMenuSkeleton({
  mode = DEFAULT_MODE,
  className,
  showIcon = false,
  ...props
}: SidebarMenuSkeletonProps) {
  const [width] = React.useState(() => {
    return `${Math.floor(Math.random() * 40) + 50}%`
  })

  if (mode === "primitive") {
    return (
      <div className={className} {...props}>
        {showIcon ? <Skeleton mode={mode} /> : null}
        <Skeleton
          mode={mode}
          style={
            {
              width,
            } as React.CSSProperties
          }
        />
      </div>
    )
  }

  return (
    <div
      data-slot="sidebar-menu-skeleton"
      data-sidebar="menu-skeleton"
      className={cn(sidebarMenuSkeletonClassName, className)}
      {...props}
    >
      {showIcon && (
        <Skeleton
          mode={mode}
          className={sidebarClassNames.slot6}
          data-sidebar="menu-skeleton-icon"
        />
      )}
      <Skeleton
        mode={mode}
        className={sidebarClassNames.slot7}
        data-sidebar="menu-skeleton-text"
        style={
          {
            "--skeleton-width": width,
          } as React.CSSProperties
        }
      />
    </div>
  )
}

function SidebarMenuSub({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarMenuSubProps) {
  if (mode === "primitive") {
    return <ul className={className} {...props} />
  }

  return (
    <ul
      data-slot="sidebar-menu-sub"
      data-sidebar="menu-sub"
      className={cn(sidebarMenuSubClassName, className)}
      {...props}
    />
  )
}

function SidebarMenuSubItem({
  mode = DEFAULT_MODE,
  className,
  ...props
}: SidebarMenuSubItemProps) {
  if (mode === "primitive") {
    return <li className={className} {...props} />
  }

  return (
    <li
      data-slot="sidebar-menu-sub-item"
      data-sidebar="menu-sub-item"
      className={cn(sidebarMenuSubItemClassName, className)}
      {...props}
    />
  )
}

function SidebarMenuSubButton({
  mode = DEFAULT_MODE,
  asChild = false,
  size = "md",
  isActive = false,
  className,
  ...props
}: SidebarMenuSubButtonProps) {
  const Comp = asChild ? Slot.Root : "a"

  if (mode === "primitive") {
    return <Comp className={className} {...props} />
  }

  return (
    <Comp
      data-slot="sidebar-menu-sub-button"
      data-sidebar="menu-sub-button"
      data-size={size}
      data-active={isActive}
      className={cn(sidebarMenuSubButtonVariants({ size }), className)}
      {...props}
    />
  )
}

export {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInput,
  SidebarInset,
  SidebarMenu,
  SidebarMenuAction,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSkeleton,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  SidebarProvider,
  SidebarRail,
  SidebarSeparator,
  SidebarTrigger,
}
