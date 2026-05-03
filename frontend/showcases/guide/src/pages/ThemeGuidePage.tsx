import {
  useEffect,
  useId,
  useRef,
  useState,
  type CSSProperties,
} from "react"
import {
  Button,
  Badge,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  IconButton,
  Input,
  Switch,
  toast,
} from "@workspace/ui-components"
import {
  applyThemeOverrides,
  clearThemeOverrides,
  loadThemeOverrides,
  saveThemeOverrides,
  type ThemeDraft,
  type ThemeMode,
} from "@/theme/themeOverrides"

type TokenDefinition = {
  key: string
  label: string
  description: string
}

const PREVIEWABLE_TOKEN_KEYS = new Set<string>([
  "primary",
  "primary-outline",
  "primary-foreground",
  "secondary",
  "secondary-foreground",
  "muted",
  "muted-foreground",
  "accent",
  "accent-foreground",
  "border",
  "destructive",
  "info",
  "success",
  "warning",
  "error",
  "input",
  "ring",
  "chart-1",
  "chart-2",
  "chart-3",
  "chart-4",
  "chart-5",
  "select",
  "select-foreground",
  "select-item-hover",
  "select-item-hover-foreground",
  "select-border",
  "select-ring",
  "sidebar",
  "sidebar-foreground",
  "sidebar-primary",
  "sidebar-primary-foreground",
  "sidebar-accent",
  "sidebar-accent-foreground",
  "sidebar-border",
  "sidebar-ring",
])

const CORE_TOKEN_DEFINITIONS: readonly TokenDefinition[] = [
  { key: "background", label: "Background", description: "页面背景主色。" },
  { key: "foreground", label: "Foreground", description: "全局正文文本颜色。" },
  { key: "surface", label: "Surface", description: "大面积表面层背景。" },
  {
    key: "surface-foreground",
    label: "Surface Foreground",
    description: "表面层上的文本或图标颜色。",
  },
  {
    key: "surface-component",
    label: "Surface Component",
    description: "组件承载层或更高一层的表面底色。",
  },
  { key: "card", label: "Card", description: "Card 组件背景。" },
  {
    key: "card-foreground",
    label: "Card Foreground",
    description: "Card 组件文本颜色。",
  },
  { key: "popover", label: "Popover", description: "浮层背景色。" },
  {
    key: "popover-foreground",
    label: "Popover Foreground",
    description: "浮层文本颜色。",
  },
  { key: "primary", label: "Primary", description: "主按钮、主强调色。" },
  {
    key: "primary-outline",
    label: "Primary Outline",
    description: "主色描边和弱态边框。",
  },
  {
    key: "primary-foreground",
    label: "Primary Foreground",
    description: "主色块上的文本颜色。",
  },
  { key: "secondary", label: "Secondary", description: "次级表面或按钮背景。" },
  {
    key: "secondary-foreground",
    label: "Secondary Foreground",
    description: "次级表面上的文本颜色。",
  },
  { key: "muted", label: "Muted", description: "弱化背景、说明区块。" },
  {
    key: "muted-foreground",
    label: "Muted Foreground",
    description: "弱化说明文本。",
  },
  { key: "accent", label: "Accent", description: "交互 hover、高亮辅助色。" },
  {
    key: "accent-foreground",
    label: "Accent Foreground",
    description: "accent 背景上的文本颜色。",
  },
  { key: "border", label: "Border", description: "默认边框色。" },
  {
    key: "destructive",
    label: "Destructive",
    description: "危险操作与错误强调。",
  },
  { key: "info", label: "Info", description: "信息态颜色。" },
  { key: "success", label: "Success", description: "成功态颜色。" },
  { key: "warning", label: "Warning", description: "警告态颜色。" },
  { key: "error", label: "Error", description: "错误态颜色。" },
] as const

const COMPONENT_TOKEN_DEFINITIONS: readonly TokenDefinition[] = [
  { key: "input", label: "Input", description: "输入框边框/背景语义色。" },
  { key: "ring", label: "Ring", description: "focus ring 颜色。" },
  { key: "chart-1", label: "Chart 1", description: "图表主色 1。" },
  { key: "chart-2", label: "Chart 2", description: "图表主色 2。" },
  { key: "chart-3", label: "Chart 3", description: "图表主色 3。" },
  { key: "chart-4", label: "Chart 4", description: "图表主色 4。" },
  { key: "chart-5", label: "Chart 5", description: "图表主色 5。" },
  { key: "select", label: "Select", description: "下拉面板背景色。" },
  {
    key: "select-foreground",
    label: "Select Foreground",
    description: "下拉面板文本颜色。",
  },
  {
    key: "select-item-hover",
    label: "Select Item Hover",
    description: "下拉选项 hover / highlighted 背景色。",
  },
  {
    key: "select-item-hover-foreground",
    label: "Select Item Hover Foreground",
    description: "下拉选项 hover / highlighted 文本色。",
  },
  {
    key: "select-border",
    label: "Select Border",
    description: "下拉触发器与面板边框色。",
  },
  {
    key: "select-ring",
    label: "Select Ring",
    description: "下拉面板 ring 颜色。",
  },
  { key: "sidebar", label: "Sidebar", description: "侧边栏背景色。" },
  {
    key: "sidebar-foreground",
    label: "Sidebar Foreground",
    description: "侧边栏文字颜色。",
  },
  {
    key: "sidebar-primary",
    label: "Sidebar Primary",
    description: "侧边栏主强调色。",
  },
  {
    key: "sidebar-primary-foreground",
    label: "Sidebar Primary Foreground",
    description: "侧边栏主强调文本色。",
  },
  {
    key: "sidebar-accent",
    label: "Sidebar Accent",
    description: "侧边栏次级强调色。",
  },
  {
    key: "sidebar-accent-foreground",
    label: "Sidebar Accent Foreground",
    description: "侧边栏次级强调文本色。",
  },
  {
    key: "sidebar-border",
    label: "Sidebar Border",
    description: "侧边栏分隔边框色。",
  },
  {
    key: "sidebar-ring",
    label: "Sidebar Ring",
    description: "侧边栏聚焦 ring 颜色。",
  },
] as const

const ALL_TOKEN_DEFINITIONS = [
  ...CORE_TOKEN_DEFINITIONS,
  ...COMPONENT_TOKEN_DEFINITIONS,
] as const

const DEFAULT_TOKENS: ThemeDraft = {
  light: {
    background: "oklch(0.99 0.002 247)",
    foreground: "oklch(0.27 0.02 256)",
    surface: "color(display-p3 0.925 0.925 0.925)",
    "surface-foreground": "oklch(0.27 0.02 256)",
    "surface-component": "color(display-p3 0.825 0.825 0.825)",
    card: "color(display-p3 0.925 0.925 0.925)",
    "card-foreground": "oklch(0.27 0.02 256)",
    popover: "color(display-p3 0.925 0.925 0.925)",
    "popover-foreground": "oklch(0.27 0.02 256)",
    primary: "color(display-p3 0.142 0.343 0.791)",
    "primary-outline": "color(display-p3 0.151 0.394 0.652)",
    "primary-foreground": "oklch(0.99 0 0)",
    secondary: "color(display-p3 0.88 0.9 0.94)",
    "secondary-foreground": "color(display-p3 0.25 0.28 0.32)",
    muted: "oklch(0.96 0.008 247)",
    "muted-foreground": "oklch(0.54 0.02 254)",
    accent: "oklch(0.94 0.015 250)",
    "accent-foreground": "oklch(0.31 0.02 256)",
    border: "oklch(0.89 0.01 250)",
    destructive: "oklch(0.62 0.22 28)",
    info: "color(display-p3 0.3 0.52 0.9)",
    success: "color(display-p3 0.42 0.7 0.35)",
    warning: "color(display-p3 0.7 0.45 0.22)",
    error: "color(display-p3 0.7 0.35 0.33)",
    input: "oklch(0.9 0.012 250)",
    ring: "oklch(0.62 0.18 258)",
    "chart-1": "oklch(0.63 0.2 258)",
    "chart-2": "oklch(0.72 0.16 200)",
    "chart-3": "oklch(0.69 0.18 145)",
    "chart-4": "oklch(0.73 0.17 85)",
    "chart-5": "oklch(0.65 0.2 20)",
    select: "color(display-p3 0.925 0.925 0.925)",
    "select-foreground": "oklch(0.27 0.02 256)",
    "select-item-hover": "oklch(0.57 0.19 258)",
    "select-item-hover-foreground": "oklch(0.99 0 0)",
    "select-border": "oklch(0.89 0.01 250)",
    "select-ring": "oklch(0.62 0.18 258)",
    sidebar: "oklch(0.975 0.006 250)",
    "sidebar-foreground": "oklch(0.27 0.02 256)",
    "sidebar-primary": "oklch(0.57 0.19 258)",
    "sidebar-primary-foreground": "oklch(0.99 0 0)",
    "sidebar-accent": "oklch(0.94 0.015 250)",
    "sidebar-accent-foreground": "oklch(0.31 0.02 256)",
    "sidebar-border": "oklch(0.89 0.01 250)",
    "sidebar-ring": "oklch(0.62 0.18 258)",
  },
  dark: {
    background: "oklch(0.2 0.012 255)",
    foreground: "oklch(0.95 0.008 250)",
    surface: "color(display-p3 0.163 0.17 0.167)",
    "surface-foreground": "color(display-p3 0.878 0.879 0.879)",
    "surface-component": "color(display-p3 0.213 0.22 0.217)",
    card: "color(display-p3 0.163 0.17 0.167)",
    "card-foreground": "color(display-p3 0.878 0.879 0.879)",
    popover: "color(display-p3 0.163 0.17 0.167)",
    "popover-foreground": "color(display-p3 0.878 0.879 0.879)",
    primary: "color(display-p3 0.142 0.343 0.791)",
    "primary-outline": "color(display-p3 0.1 0.55 1)",
    "primary-foreground": "white",
    secondary: "color(display-p3 0.22 0.25 0.29)",
    "secondary-foreground": "oklch(0.93 0.01 258)",
    muted: "oklch(0.26 0.012 255)",
    "muted-foreground": "oklch(0.74 0.012 250)",
    accent: "oklch(0.31 0.02 255)",
    "accent-foreground": "oklch(0.95 0.008 250)",
    destructive: "oklch(0.71 0.18 24)",
    border: "oklch(0.38 0.01 252 / 65%)",
    info: "color(display-p3 0.242 0.543 0.991)",
    success: "color(display-p3 0.3954 0.7694 0.4)",
    warning: "color(display-p3 0.9404 0.5755 0.2617)",
    error: "color(display-p3 0.9209 0.2964 0.2741)",
    input: "oklch(0.33 0.01 252 / 70%)",
    ring: "oklch(0.69 0.15 258)",
    "chart-1": "oklch(0.69 0.15 258)",
    "chart-2": "oklch(0.72 0.15 205)",
    "chart-3": "oklch(0.73 0.15 150)",
    "chart-4": "oklch(0.75 0.14 95)",
    "chart-5": "oklch(0.7 0.18 30)",
    select: "color(display-p3 0.163 0.17 0.167)",
    "select-foreground": "color(display-p3 0.878 0.879 0.879)",
    "select-item-hover": "oklch(0.69 0.15 258)",
    "select-item-hover-foreground": "oklch(0.2 0.01 255)",
    "select-border": "oklch(0.38 0.01 252 / 65%)",
    "select-ring": "oklch(0.69 0.15 258)",
    sidebar: "oklch(0.22 0.014 255)",
    "sidebar-foreground": "oklch(0.95 0.008 250)",
    "sidebar-primary": "oklch(0.69 0.15 258)",
    "sidebar-primary-foreground": "oklch(0.2 0.01 255)",
    "sidebar-accent": "oklch(0.31 0.02 255)",
    "sidebar-accent-foreground": "oklch(0.95 0.008 250)",
    "sidebar-border": "oklch(0.38 0.01 252 / 65%)",
    "sidebar-ring": "oklch(0.69 0.15 258)",
  },
}

function mergeDraftWithDefaults(
  draft: ThemeDraft | null | undefined
): ThemeDraft {
  return {
    light: {
      ...DEFAULT_TOKENS.light,
      ...(draft?.light ?? {}),
    },
    dark: {
      ...DEFAULT_TOKENS.dark,
      ...(draft?.dark ?? {}),
    },
  }
}

function getResolvedDocumentTheme(): ThemeMode {
  if (typeof document === "undefined") {
    return "light"
  }

  const attr = document.documentElement.getAttribute("data-theme")
  return attr === "dark" ? "dark" : "light"
}

function resolveColorInputValue(value: string) {
  if (typeof document === "undefined") {
    return "#000000"
  }

  const sample = document.createElement("span")
  sample.style.color = ""
  sample.style.color = value

  if (!sample.style.color) {
    return "#000000"
  }

  document.body.appendChild(sample)
  const resolved = window.getComputedStyle(sample).color
  sample.remove()

  const match = resolved.match(
    /rgba?\(\s*(\d{1,3})[\s,]+(\d{1,3})[\s,]+(\d{1,3})/i
  )

  if (!match) {
    return "#000000"
  }

  const toHex = (channel: string) =>
    Number(channel).toString(16).padStart(2, "0")

  return `#${toHex(match[1])}${toHex(match[2])}${toHex(match[3])}`
}

function buildThemeCss(mode: ThemeMode, values: ThemeDraft[ThemeMode]) {
  const selector =
    mode === "light" ? ":root" : 'html[data-theme="dark"],\n:root.dark'

  const lines = ALL_TOKEN_DEFINITIONS.map(
    (token) => `  --${token.key}: ${values[token.key]};`
  ).join("\n")

  return `${selector} {\n${lines}\n}`
}

function buildPreviewTokenStyle(modeValues: ThemeDraft[ThemeMode]) {
  return ALL_TOKEN_DEFINITIONS.reduce<Record<string, string>>((style, token) => {
    style[`--${token.key}`] = modeValues[token.key]
    return style
  }, {})
}

function getPreviewTitle(tokenKey: string) {
  if (tokenKey.startsWith("chart-")) {
    return "图表色板预览"
  }

  if (tokenKey.startsWith("select")) {
    return "下拉选择预览"
  }

  if (tokenKey.startsWith("sidebar")) {
    return "侧边导航预览"
  }

  if (tokenKey === "input" || tokenKey === "ring") {
    return "表单交互预览"
  }

  if (tokenKey === "info" || tokenKey === "success" || tokenKey === "warning" || tokenKey === "error") {
    return "状态语义预览"
  }

  return "组件语义预览"
}

function TokenPreviewPanel({
  token,
  draft,
  activeMode,
}: {
  token: TokenDefinition
  draft: ThemeDraft
  activeMode: ThemeMode
}) {
  const previewStyle = buildPreviewTokenStyle(draft[activeMode]) as CSSProperties

  if (!PREVIEWABLE_TOKEN_KEYS.has(token.key)) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>{token.label}</CardTitle>
          <CardDescription>
            这是基础语义 token，主要作用在整体基底层，不提供单独组件预览。
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="rounded-xl border border-(--border) bg-(--surface) p-4 text-sm text-(--muted-foreground)">
            建议直接观察整页背景、文本和表面层的整体变化。
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <div style={previewStyle}>
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between gap-3">
            <div>
              <CardTitle>{getPreviewTitle(token.key)}</CardTitle>
              <CardDescription>
                当前预览聚焦 `{token.label}`，右侧示例会直接消费当前主题变量。
              </CardDescription>
            </div>
            <Badge variant="outline">{token.key}</Badge>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {token.key === "primary" ||
            token.key === "primary-outline" ||
            token.key === "primary-foreground" ? (
              <div className="rounded-xl border border-(--border) bg-(--surface) p-4">
                <div className="mb-3 flex flex-wrap gap-2">
                  <Button variant="primary">Primary Action</Button>
                  <Button variant="outline">Outline Action</Button>
                  <Badge>Live</Badge>
                  <IconButton variant="primary" label="favorite">
                    <span className="text-sm">★</span>
                  </IconButton>
                </div>
                <div className="text-sm text-(--muted-foreground)">
                  这里的主按钮、描边按钮、badge 和 icon action 都直接消费 primary 相关语义。
                </div>
              </div>
            ) : null}

            {token.key === "secondary" || token.key === "secondary-foreground" ? (
              <div className="rounded-xl border border-(--border) bg-(--surface) p-4">
                <div className="mb-3 flex flex-wrap gap-2">
                  <Button variant="secondary">Secondary</Button>
                  <div className="rounded-lg bg-(--secondary) px-3 py-2 text-sm text-(--secondary-foreground)">
                    Secondary Surface
                  </div>
                </div>
                <div className="text-sm text-(--muted-foreground)">
                  用来判断次级按钮和弱强调表面的层级是否合适。
                </div>
              </div>
            ) : null}

            {token.key === "muted" || token.key === "muted-foreground" ? (
              <div className="rounded-xl border border-(--border) bg-(--muted) p-4">
                <div className="text-sm font-medium text-(--foreground)">Muted block</div>
                <div className="mt-1 text-sm text-(--muted-foreground)">
                  说明文字、弱提示、次要信息会落在这一组语义上。
                </div>
              </div>
            ) : null}

            {token.key === "accent" || token.key === "accent-foreground" ? (
              <div className="rounded-xl border border-(--border) bg-(--surface) p-4">
                <div className="mb-3 flex flex-wrap gap-2">
                  <div className="rounded-lg bg-(--accent) px-3 py-2 text-sm text-(--accent-foreground)">
                    Hover Surface
                  </div>
                  <IconButton label="accent action">
                    <span className="text-sm">✦</span>
                  </IconButton>
                </div>
                <div className="text-sm text-(--muted-foreground)">
                  accent 更适合 hover、选中背景和轻强调区块。
                </div>
              </div>
            ) : null}

            {token.key === "border" ? (
              <div className="grid gap-3">
                <div className="rounded-xl border border-(--border) bg-(--surface) p-4">Panel Border</div>
                <div className="rounded-xl border border-(--border) bg-(--card) p-4">Card Border</div>
              </div>
            ) : null}

            {token.key === "destructive" ||
            token.key === "info" ||
            token.key === "success" ||
            token.key === "warning" ||
            token.key === "error" ? (
              <div className="grid gap-3">
                <div className="flex flex-wrap gap-2">
                  <Button variant="destructive">Delete</Button>
                  <IconButton variant="destructive" label="delete">
                    <span className="text-sm">⌫</span>
                  </IconButton>
                </div>
                <div className="grid gap-2">
                  <div className="rounded-lg px-3 py-2 text-sm" style={{ background: "color-mix(in oklab, var(--info) 14%, transparent)", color: "var(--info)" }}>
                    Info notice
                  </div>
                  <div className="rounded-lg px-3 py-2 text-sm" style={{ background: "color-mix(in oklab, var(--success) 14%, transparent)", color: "var(--success)" }}>
                    Success notice
                  </div>
                  <div className="rounded-lg px-3 py-2 text-sm" style={{ background: "color-mix(in oklab, var(--warning) 14%, transparent)", color: "var(--warning)" }}>
                    Warning notice
                  </div>
                  <div className="rounded-lg px-3 py-2 text-sm" style={{ background: "color-mix(in oklab, var(--error) 14%, transparent)", color: "var(--error)" }}>
                    Error notice
                  </div>
                </div>
              </div>
            ) : null}

            {token.key === "input" || token.key === "ring" ? (
              <div className="grid gap-3">
                <Input value="Theme input preview" onValueChange={() => {}} />
                <div className="rounded-xl border border-(--input) bg-(--background) p-3 shadow-[0_0_0_3px_color-mix(in_oklab,var(--ring)_45%,transparent)]">
                  <div className="text-sm">Focused field shell</div>
                </div>
                <div className="flex items-center gap-3">
                  <Switch checked />
                  <span className="text-sm text-(--muted-foreground)">Switch focus / active contrast</span>
                </div>
              </div>
            ) : null}

            {token.key.startsWith("chart-") ? (
              <div className="space-y-4">
                <div className="grid grid-cols-5 gap-2">
                  {["chart-1", "chart-2", "chart-3", "chart-4", "chart-5"].map((key) => (
                    <div key={key} className="space-y-2 text-center text-xs text-(--muted-foreground)">
                      <div className="h-16 rounded-lg" style={{ background: `var(--${key})` }} />
                      <div>{key}</div>
                    </div>
                  ))}
                </div>
                <div className="flex items-end gap-2 rounded-xl border border-(--border) bg-(--surface) p-4">
                  {[36, 54, 48, 68, 42].map((height, index) => (
                    <div
                      key={height}
                      className="flex-1 rounded-t-md"
                      style={{
                        height,
                        background: `var(--chart-${index + 1})`,
                      }}
                    />
                  ))}
                </div>
              </div>
            ) : null}

            {token.key.startsWith("select") ? (
              <div className="rounded-xl border border-(--select-border) bg-(--surface) p-4">
                <div className="w-72 rounded-lg border border-(--select-border) bg-transparent text-(--foreground)">
                  <div className="flex items-center justify-between gap-3 px-3 py-2 text-sm">
                    <span>Status</span>
                    <span className="inline-flex size-6 items-center justify-center rounded-full bg-(--surface) text-(--select-foreground)">
                      ▾
                    </span>
                  </div>
                </div>
                <div className="mt-3 w-72 rounded-lg border border-(--select-border) bg-(--select) p-1 text-(--select-foreground) shadow-[0_0_0_1px_color-mix(in_oklab,var(--select-ring)_10%,transparent)]">
                  <div className="px-1.5 py-1 text-xs text-(--muted-foreground)">Options</div>
                  <div className="rounded-md px-3 py-2 text-sm">Draft</div>
                  <div className="rounded-md bg-(--select-item-hover) px-3 py-2 text-sm text-(--select-item-hover-foreground)">
                    Published
                  </div>
                  <div className="rounded-md px-3 py-2 text-sm">Archived</div>
                </div>
              </div>
            ) : null}

            {token.key.startsWith("sidebar") ? (
              <div className="rounded-xl border border-(--sidebar-border) bg-(--sidebar) p-4 text-(--sidebar-foreground)">
                <div className="flex items-center justify-between">
                  <div className="text-sm font-semibold">Workspace</div>
                  <IconButton variant="primary" label="sidebar primary">
                    <span className="text-sm">+</span>
                  </IconButton>
                </div>
                <div className="mt-4 space-y-2">
                  <div className="rounded-lg bg-(--sidebar-accent) px-3 py-2 text-sm text-(--sidebar-accent-foreground)">
                    Active item
                  </div>
                  <div className="rounded-lg px-3 py-2 text-sm">Default item</div>
                  <div
                    className="rounded-lg px-3 py-2 text-sm"
                    style={{
                      background: "var(--sidebar-primary)",
                      color: "var(--sidebar-primary-foreground)",
                    }}
                  >
                    Primary shortcut
                  </div>
                </div>
              </div>
            ) : null}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function ThemeTokenField({
  token,
  activeMode,
  value,
  onChange,
  selected,
  onSelect,
}: {
  token: TokenDefinition
  activeMode: ThemeMode
  value: string
  onChange: (value: string) => void
  selected: boolean
  onSelect: () => void
}) {
  const colorInputId = useId()
  const colorInputRef = useRef<HTMLInputElement | null>(null)
  const triggerRef = useRef<HTMLButtonElement | null>(null)
  const swatchRef = useRef<HTMLSpanElement | null>(null)

  const getHexFromRenderedSwatch = () => {
    const swatch = swatchRef.current
    if (!swatch || typeof window === "undefined") {
      return resolveColorInputValue(value)
    }

    const resolved = window.getComputedStyle(swatch).backgroundColor
    const match = resolved.match(
      /rgba?\(\s*(\d{1,3})[\s,]+(\d{1,3})[\s,]+(\d{1,3})/i
    )

    if (!match) {
      return resolveColorInputValue(value)
    }

    const toHex = (channel: string) =>
      Number(channel).toString(16).padStart(2, "0")

    return `#${toHex(match[1])}${toHex(match[2])}${toHex(match[3])}`
  }

  const positionColorInputNearTrigger = () => {
    const input = colorInputRef.current
    const trigger = triggerRef.current
    if (!input || !trigger) {
      return
    }

    input.value = getHexFromRenderedSwatch()

    const rect = trigger.getBoundingClientRect()
    input.style.left = `${Math.round(rect.left + rect.width / 2)}px`
    input.style.top = `${Math.round(rect.top + rect.height / 2)}px`
  }

  const openColorPicker = () => {
    const input = colorInputRef.current
    if (!input) {
      return
    }

    positionColorInputNearTrigger()

    if ("showPicker" in input && typeof input.showPicker === "function") {
      input.showPicker()
      return
    }

    input.click()
  }

  return (
    <div
      className="rounded-xl border bg-(--card) p-3 transition-shadow"
      style={{
        borderColor: selected ? "var(--primary)" : "var(--border)",
        boxShadow: selected
          ? "0 0 0 1px color-mix(in oklab, var(--primary) 55%, transparent)"
          : "none",
      }}
      onClick={onSelect}
      onFocusCapture={onSelect}
    >
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="text-sm font-medium">{token.label}</div>
          <div className="mt-1 text-xs text-(--muted-foreground)">
            {token.description}
          </div>
        </div>
        <button
          type="button"
          ref={triggerRef}
          className="block size-14  shrink-0 cursor-pointer rounded-2xl border border-(--border) p-3"
          aria-label={`${token.label} color picker`}
          title={`选择 ${activeMode} ${token.label} 颜色`}
          onClick={openColorPicker}
        >
          <span
            ref={swatchRef}
            className="block size-full  rounded-md border border-black/10"
            style={{ background: value }}
          />
        </button>
      </div>
      <div className="mt-3 flex items-center gap-3">
        <input
          id={colorInputId}
          ref={colorInputRef}
          type="color"
          value={resolveColorInputValue(value)}
          tabIndex={-1}
          aria-hidden="true"
          className="pointer-events-none fixed z-[-1] size-px  opacity-0"
          onChange={(event) => onChange(event.target.value)}
        />
        <div className="min-w-0 flex-1">
          <Input value={value} onValueChange={onChange} />
        </div>
      </div>
    </div>
  )
}

export default function ThemeGuidePage() {
  const [draft, setDraft] = useState<ThemeDraft>(() =>
    mergeDraftWithDefaults(loadThemeOverrides())
  )
  const [editorShellMode, setEditorShellMode] = useState<ThemeMode>(() =>
    getResolvedDocumentTheme()
  )
  const [selectedTokenKey, setSelectedTokenKey] = useState("primary")
  const activeMode = editorShellMode
  const selectedToken =
    ALL_TOKEN_DEFINITIONS.find((token) => token.key === selectedTokenKey) ??
    ALL_TOKEN_DEFINITIONS[0]

  useEffect(() => {
    if (typeof document === "undefined") {
      return undefined
    }

    const root = document.documentElement
    const observer = new MutationObserver(() => {
      const resolvedTheme = getResolvedDocumentTheme()
      setEditorShellMode(resolvedTheme)
    })

    observer.observe(root, {
      attributes: true,
      attributeFilter: ["data-theme"],
    })

    return () => {
      observer.disconnect()
    }
  }, [])

  const editorShellStyle = ALL_TOKEN_DEFINITIONS.reduce((style, token) => {
    style[`--${token.key}`] = DEFAULT_TOKENS[editorShellMode][token.key]
    return style
  }, {} as Record<string, string>) as CSSProperties

  const updateToken = (mode: ThemeMode, key: string, value: string) => {
    setDraft((current) => ({
      ...current,
      [mode]: (() => {
        const nextMode = {
          ...current[mode],
          [key]: value,
        }

        const nextDraft = {
          ...current,
          [mode]: nextMode,
        }

        applyThemeOverrides(nextDraft)
        saveThemeOverrides(nextDraft)
        return nextMode
      })(),
    }))
  }

  const resetMode = (mode: ThemeMode) => {
    setDraft((current) => {
      const nextDraft = {
        ...current,
        [mode]: { ...DEFAULT_TOKENS[mode] },
      }

      applyThemeOverrides(nextDraft)
      saveThemeOverrides(nextDraft)
      return nextDraft
    })
  }

  const resetAll = () => {
    const nextDraft = mergeDraftWithDefaults(null)
    setDraft(nextDraft)
    clearThemeOverrides()
  }

  const exportCurrentThemeCss = async () => {
    const css = buildThemeCss(activeMode, draft[activeMode])
    await navigator.clipboard.writeText(css)
    toast("CSS tokens copied", {
      description: `已复制当前 ${activeMode} 主题的 token CSS。`,
    })
  }

  return (
    <div className="min-h-0 min-w-0 flex-1 overflow-auto">
      <div
        className="mx-auto flex w-full max-w-[1800px] flex-col gap-6 p-4 md:p-6"
        style={editorShellStyle}
      >
        <Card>
          <CardHeader>
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div>
                <CardTitle>Theme 配置页</CardTitle>
                <CardDescription>
                  直接修改当前 guide 项目的 light/dark 主题变量。
                </CardDescription>
              </div>
              <div className="flex flex-wrap gap-2">
                <Button variant="outline" onClick={() => void exportCurrentThemeCss()}>
                  导出当前主题 CSS
                </Button>
                <Button variant="outline" onClick={resetAll}>
                  恢复全部默认
                </Button>
                <Button variant="outline" onClick={() => resetMode(activeMode)}>
                  重置当前模式
                </Button>
              </div>
            </div>
          </CardHeader>
        </Card>

        <div
          id="semantic-tokens"
          className="grid gap-6 xl:grid-cols-[minmax(0,1.5fr)_minmax(360px,0.8fr)] 2xl:grid-cols-[minmax(0,1.65fr)_minmax(380px,0.75fr)]"
        >
          <Card>
            <CardHeader>
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <CardTitle>Semantic Tokens</CardTitle>
                  <CardDescription>
                    当前正在编辑 `{activeMode}` 主题，切换请使用 topbar。
                  </CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-6">
                <section className="space-y-4">
                  <div className="text-sm font-semibold">Core tokens</div>
                  <div className="grid gap-3 md:grid-cols-2 2xl:grid-cols-4">
                    {CORE_TOKEN_DEFINITIONS.map((token) => (
                      <ThemeTokenField
                        key={token.key}
                        token={token}
                        activeMode={activeMode}
                        value={draft[activeMode][token.key]}
                        onChange={(value) => updateToken(activeMode, token.key, value)}
                        selected={selectedTokenKey === token.key}
                        onSelect={() => setSelectedTokenKey(token.key)}
                      />
                    ))}
                  </div>
                </section>

                <section className="space-y-4">
                  <div className="text-sm font-semibold">Component tokens</div>
                  <div className="grid gap-3 md:grid-cols-2 2xl:grid-cols-4">
                    {COMPONENT_TOKEN_DEFINITIONS.map((token) => (
                      <ThemeTokenField
                        key={token.key}
                        token={token}
                        activeMode={activeMode}
                        value={draft[activeMode][token.key]}
                        onChange={(value) => updateToken(activeMode, token.key, value)}
                        selected={selectedTokenKey === token.key}
                        onSelect={() => setSelectedTokenKey(token.key)}
                      />
                    ))}
                  </div>
                </section>
              </div>
            </CardContent>
          </Card>

          <div className="xl:sticky xl:top-4 xl:self-start">
            <TokenPreviewPanel
              token={selectedToken}
              draft={draft}
              activeMode={activeMode}
            />
          </div>
        </div>
      </div>
    </div>
  )
}
