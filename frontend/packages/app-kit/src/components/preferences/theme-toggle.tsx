import { useEffect, useState } from "react"
import { MoonStar, Sun } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-core/components/button"
import { cn } from "@workspace/ui-core/lib/utils.js"
import {
  getStoredThemeMode,
  setThemeMode,
  type ThemeMode,
} from "@workspace/ui-theme"

type ResolvedTheme = "light" | "dark"

const SYSTEM_THEME_QUERY = "(prefers-color-scheme: dark)"

function getSystemTheme(): ResolvedTheme {
  if (typeof window === "undefined") return "light"
  return window.matchMedia(SYSTEM_THEME_QUERY).matches ? "dark" : "light"
}

export interface ThemeToggleProps {
  className?: string
}

export function ThemeToggle({ className }: ThemeToggleProps) {
  const { t } = useTranslation("components")
  const [mode, setMode] = useState<ThemeMode>(() => getStoredThemeMode())
  const [resolvedTheme, setResolvedTheme] = useState<ResolvedTheme>(() =>
    getSystemTheme()
  )

  useEffect(() => {
    if (typeof window === "undefined") return

    const media = window.matchMedia(SYSTEM_THEME_QUERY)

    const syncFromEnvironment = () => {
      const nextResolvedTheme = media.matches ? "dark" : "light"
      const storedMode = getStoredThemeMode()

      setResolvedTheme(nextResolvedTheme)

      if (storedMode !== "system" && storedMode === nextResolvedTheme) {
        setThemeMode("system")
        setMode("system")
        return
      }

      setMode(storedMode)
    }

    syncFromEnvironment()
    media.addEventListener("change", syncFromEnvironment)
    window.addEventListener("storage", syncFromEnvironment)

    return () => {
      media.removeEventListener("change", syncFromEnvironment)
      window.removeEventListener("storage", syncFromEnvironment)
    }
  }, [])

  const effectiveTheme = mode === "system" ? resolvedTheme : mode
  const isFollowingSystem = mode === "system"
  const isAlignedWithSystem = effectiveTheme === resolvedTheme
  const nextMode: ThemeMode =
    isFollowingSystem || isAlignedWithSystem
      ? effectiveTheme === "dark"
        ? "light"
        : "dark"
      : "system"
  const label = t(
    mode === "system" ? "header.theme.system" : `header.theme.${effectiveTheme}`
  )
  const icon =
    effectiveTheme === "dark" ? (
      <Sun
        className="size-5 text-amber-600/80 dark:text-amber-300 dark:drop-shadow-[0_0_12px_rgba(245,158,11,0.35)]"
        strokeWidth={2.4}
      />
    ) : (
      <MoonStar
        className="size-5 fill-current text-slate-600/85 dark:text-slate-100 dark:drop-shadow-[0_0_10px_rgba(96,165,250,0.16)]"
        strokeWidth={1.8}
      />
    )

  return (
    <Button
      variant="ghost"
      size="icon"
      className={cn(
        "rounded-full border border-transparent bg-transparent text-foreground shadow-none transition-colors hover:bg-[var(--surface-hover)]",
        className
      )}
      aria-label={label}
      onClick={() => {
        setMode(nextMode)
        setThemeMode(nextMode)
      }}
    >
      {icon}
    </Button>
  )
}
