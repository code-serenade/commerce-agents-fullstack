export type ThemeMode = "light" | "dark"

export type ThemeValues = Record<string, string>

export type ThemeDraft = Record<ThemeMode, ThemeValues>

const THEME_OVERRIDE_STORAGE_KEY = "guide_theme_overrides"
const THEME_OVERRIDE_STYLE_ID = "guide-theme-overrides"

function buildRule(selector: string, values: ThemeValues | undefined) {
  if (!values || Object.keys(values).length === 0) {
    return ""
  }

  const declarations = Object.entries(values)
    .map(([key, value]) => `  --${key}: ${value};`)
    .join("\n")

  return `${selector} {\n${declarations}\n}`
}

function getOverrideStyleElement() {
  const existing = document.getElementById(THEME_OVERRIDE_STYLE_ID)
  if (existing instanceof HTMLStyleElement) {
    return existing
  }

  const style = document.createElement("style")
  style.id = THEME_OVERRIDE_STYLE_ID
  document.head.appendChild(style)
  return style
}

export function loadThemeOverrides(): ThemeDraft | null {
  if (typeof window === "undefined") {
    return null
  }

  const raw = window.localStorage.getItem(THEME_OVERRIDE_STORAGE_KEY)
  if (!raw) {
    return null
  }

  try {
    const parsed = JSON.parse(raw)
    if (!parsed || typeof parsed !== "object") {
      return null
    }

    return {
      light:
        parsed.light && typeof parsed.light === "object" ? parsed.light : {},
      dark: parsed.dark && typeof parsed.dark === "object" ? parsed.dark : {},
    } as ThemeDraft
  } catch {
    return null
  }
}

export function saveThemeOverrides(draft: ThemeDraft) {
  if (typeof window === "undefined") {
    return
  }

  window.localStorage.setItem(THEME_OVERRIDE_STORAGE_KEY, JSON.stringify(draft))
}

export function clearThemeOverrides() {
  if (typeof window !== "undefined") {
    window.localStorage.removeItem(THEME_OVERRIDE_STORAGE_KEY)
  }

  if (typeof document === "undefined") {
    return
  }

  const style = document.getElementById(THEME_OVERRIDE_STYLE_ID)
  style?.remove()
}

export function applyThemeOverrides(draft: ThemeDraft) {
  if (typeof document === "undefined") {
    return
  }

  const css = [
    buildRule(":root", draft.light),
    buildRule('html[data-theme="dark"],\n:root.dark', draft.dark),
  ]
    .filter(Boolean)
    .join("\n\n")

  if (!css) {
    clearThemeOverrides()
    return
  }

  const style = getOverrideStyleElement()
  style.textContent = css
}
