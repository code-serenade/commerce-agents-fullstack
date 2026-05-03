"use client"

import { useTheme } from "next-themes"
import { Toaster as Sonner } from "sonner"
import type { ToasterProps } from "sonner"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import {
  CircleCheckIcon,
  InfoIcon,
  TriangleAlertIcon,
  OctagonXIcon,
  Loader2Icon,
} from "../../lib/icon-slots"
import { sonnerClassNames } from "./sonner.styles"
import type { SonnerClassResolver, SonnerToasterProps } from "./sonner.types"

function resolveStyledSonnerClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: SonnerClassResolver
}) {
  if (classResolver) {
    return classResolver({
      defaultClassName,
      className,
    })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

const Toaster = ({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  iconClassName,
  iconClassNameMode = "merge",
  iconClassResolver,
  loadingIconClassName,
  loadingIconClassNameMode = "merge",
  loadingIconClassResolver,
  toastClassName,
  toastClassNameMode = "merge",
  toastClassResolver,
  icons,
  toastOptions,
  ...props
}: SonnerToasterProps) => {
  const { theme = "system" } = useTheme()

  if (mode === "primitive") {
    return (
      <Sonner
        theme={theme as ToasterProps["theme"]}
        className={className}
        icons={icons}
        toastOptions={toastOptions}
        {...props}
      />
    )
  }

  const resolvedIconClassName = resolveStyledSonnerClassName({
    className: iconClassName,
    defaultClassName: sonnerClassNames.slot2,
    classNameMode: iconClassNameMode,
    classResolver: iconClassResolver,
  })

  const resolvedLoadingIconClassName = resolveStyledSonnerClassName({
    className: loadingIconClassName,
    defaultClassName: sonnerClassNames.slot3,
    classNameMode: loadingIconClassNameMode,
    classResolver: loadingIconClassResolver,
  })

  const resolvedToastClassName = resolveStyledSonnerClassName({
    className: cn(toastOptions?.classNames?.toast, toastClassName),
    defaultClassName: sonnerClassNames.slot4,
    classNameMode: toastClassNameMode,
    classResolver: toastClassResolver,
  })

  return (
    <Sonner
      theme={theme as ToasterProps["theme"]}
      className={resolveStyledSonnerClassName({
        className,
        defaultClassName: sonnerClassNames.slot1,
        classNameMode,
        classResolver,
      })}
      icons={{
        success: <CircleCheckIcon className={resolvedIconClassName} />,
        info: <InfoIcon className={resolvedIconClassName} />,
        warning: <TriangleAlertIcon className={resolvedIconClassName} />,
        error: <OctagonXIcon className={resolvedIconClassName} />,
        loading: <Loader2Icon className={resolvedLoadingIconClassName} />,
        ...icons,
      }}
      style={
        {
          "--normal-bg": "var(--popover)",
          "--normal-text": "var(--popover-foreground)",
          "--normal-border": "var(--border)",
          "--border-radius": "var(--radius)",
        } as React.CSSProperties
      }
      toastOptions={{
        ...toastOptions,
        classNames: {
          ...toastOptions?.classNames,
          toast: resolvedToastClassName,
        },
      }}
      {...props}
    />
  )
}

export { Toaster }
