import type { ReactNode } from "react"
import {
  ArrowRight,
  Bell,
  Download,
  Heart,
  LoaderCircle,
  Mail,
  Pencil,
  Plus,
  Search,
  Settings2,
  Trash2,
} from "lucide-react"
import {
  Badge,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  IconButton,
} from "@workspace/ui-components"

const BUTTON_VARIANTS = [
  "default",
  "primary",
  "secondary",
  "destructive",
  "outline",
  "ghost",
  "link",
] as const

const BUTTON_SIZES = ["xs", "sm", "default", "lg"] as const

const ICON_BUTTON_VARIANTS = [
  "default",
  "primary",
  "info",
  "success",
  "warning",
  "destructive",
] as const

const BUTTON_ICON_MAP: Record<(typeof BUTTON_VARIANTS)[number], ReactNode> = {
  default: <Plus className="size-4" />,
  primary: <ArrowRight className="size-4" />,
  secondary: <Mail className="size-4" />,
  destructive: <Trash2 className="size-4" />,
  outline: <Download className="size-4" />,
  ghost: <Search className="size-4" />,
  link: <ArrowRight className="size-4" />,
}

const ICON_BUTTON_ICON_MAP: Record<
  (typeof ICON_BUTTON_VARIANTS)[number],
  ReactNode
> = {
  default: <Bell className="size-4" />,
  primary: <Heart className="size-4" />,
  info: <Mail className="size-4" />,
  success: <Download className="size-4" />,
  warning: <Settings2 className="size-4" />,
  destructive: <Trash2 className="size-4" />,
}

export default function ButtonGuidePage() {
  return (
    <div className="min-h-0 flex-1 overflow-auto">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-6 p-4 md:p-6">
        <section
          id="overview"
          className="overflow-hidden rounded-[28px] border border-border/60 bg-[linear-gradient(135deg,color-mix(in_oklab,var(--primary)_16%,var(--card))_0%,var(--card)_45%,color-mix(in_oklab,var(--accent)_18%,var(--card))_100%)]"
        >
          <div className="flex flex-col gap-6 p-6 md:p-8">
            <div className="flex flex-wrap items-center gap-3">
              <Badge variant="outline">UI Components</Badge>
              <Badge variant="outline">Button</Badge>
              <Badge variant="outline">IconButton</Badge>
            </div>
            <div className="max-w-3xl space-y-3">
              <h1 className="text-3xl font-semibold tracking-tight md:text-4xl">
                Button Guide
              </h1>
              <p className="text-sm/6  text-muted-foreground md:text-base">
                这个页面把当前库里所有对外暴露的按钮形态集中展示出来。普通
                Button 会覆盖全部 variant 和常用 size，IconButton 也单独列出。
              </p>
            </div>
            <div className="grid gap-3 md:grid-cols-3">
              <StatCard title="Button variants" value="7" />
              <StatCard title="Button sizes" value="4 + icon" />
              <StatCard title="IconButton variants" value="6" />
            </div>
          </div>
        </section>

        <section id="button-variants">
          <Card>
            <CardHeader>
              <CardTitle>Button Variants</CardTitle>
              <CardDescription>
                每一行是一个 variant，右侧补了 disabled 状态和 icon 组合，方便直接看语义差异。
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {BUTTON_VARIANTS.map((variant) => (
                  <div
                    key={variant}
                    className="grid gap-4 rounded-2xl border border-border/60 bg-card/70 p-4 lg:grid-cols-[140px_minmax(0,1fr)] lg:items-center"
                  >
                    <div className="space-y-1">
                      <p className="text-sm font-medium capitalize">{variant}</p>
                      <p className="text-xs text-muted-foreground">
                        variant="{variant}"
                      </p>
                    </div>
                    <div className="flex flex-wrap items-center gap-3">
                      <Button variant={variant}>{getButtonLabel(variant)}</Button>
                      <Button variant={variant} disabled>
                        Disabled
                      </Button>
                      <Button variant={variant}>
                        {BUTTON_ICON_MAP[variant]}
                        <span>{getIconLabel(variant)}</span>
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </section>

        <section id="button-sizes">
          <Card>
            <CardHeader>
              <CardTitle>Button Sizes</CardTitle>
              <CardDescription>
                常规按钮统一用 primary 展示尺寸差异，额外补一个 icon size 说明它的占位效果。
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="grid gap-4 rounded-2xl border border-border/60 bg-card/70 p-4 lg:grid-cols-[140px_minmax(0,1fr)] lg:items-center">
                  <div className="space-y-1">
                    <p className="text-sm font-medium">Text sizes</p>
                    <p className="text-xs text-muted-foreground">
                      size="xs" | "sm" | "default" | "lg"
                    </p>
                  </div>
                  <div className="flex flex-wrap items-center gap-3">
                    {BUTTON_SIZES.map((size) => (
                      <Button key={size} variant="primary" size={size}>
                        {size}
                      </Button>
                    ))}
                  </div>
                </div>

                <div className="grid gap-4 rounded-2xl border border-border/60 bg-card/70 p-4 lg:grid-cols-[140px_minmax(0,1fr)] lg:items-center">
                  <div className="space-y-1">
                    <p className="text-sm font-medium">Icon size</p>
                    <p className="text-xs text-muted-foreground">size="icon"</p>
                  </div>
                  <div className="flex flex-wrap items-center gap-3">
                    <Button variant="primary" size="icon">
                      <Plus className="size-4" />
                    </Button>
                    <Button variant="outline" size="icon">
                      <Pencil className="size-4" />
                    </Button>
                    <Button variant="ghost" size="icon" disabled>
                      <LoaderCircle className="size-4 animate-spin" />
                    </Button>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </section>

        <section id="button-layout">
          <Card>
            <CardHeader>
              <CardTitle>Layout States</CardTitle>
              <CardDescription>
                除了视觉变体，还补了全宽和组合操作的典型用法，便于页面落地时直接对照。
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid gap-4 lg:grid-cols-2">
                <div className="rounded-2xl border border-border/60 bg-card/70 p-4">
                  <p className="mb-3 text-sm font-medium">Full width</p>
                  <div className="space-y-3">
                    <Button variant="primary" fullWidth>
                      Continue to Checkout
                    </Button>
                    <Button variant="outline" fullWidth>
                      Save Draft
                    </Button>
                  </div>
                </div>
                <div className="rounded-2xl border border-border/60 bg-card/70 p-4">
                  <p className="mb-3 text-sm font-medium">Action cluster</p>
                  <div className="flex flex-wrap items-center gap-3">
                    <Button variant="secondary">
                      <Mail className="size-4" />
                      <span>Invite</span>
                    </Button>
                    <Button variant="primary">
                      <Plus className="size-4" />
                      <span>Create Workspace</span>
                    </Button>
                    <Button variant="destructive">
                      <Trash2 className="size-4" />
                      <span>Delete</span>
                    </Button>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </section>

        <section id="icon-button">
          <Card>
            <CardHeader>
              <CardTitle>IconButton Variants</CardTitle>
              <CardDescription>
                IconButton 目前固定为 ghost + icon 尺寸，差异主要来自语义色。这里把全部 variant 都列出来。
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {ICON_BUTTON_VARIANTS.map((variant) => (
                  <div
                    key={variant}
                    className="grid gap-4 rounded-2xl border border-border/60 bg-card/70 p-4 lg:grid-cols-[140px_minmax(0,1fr)] lg:items-center"
                  >
                    <div className="space-y-1">
                      <p className="text-sm font-medium capitalize">{variant}</p>
                      <p className="text-xs text-muted-foreground">
                        variant="{variant}"
                      </p>
                    </div>
                    <div className="flex flex-wrap items-center gap-3">
                      <IconButton variant={variant} label={`${variant} action`}>
                        {ICON_BUTTON_ICON_MAP[variant]}
                      </IconButton>
                      <IconButton
                        variant={variant}
                        label={`${variant} action disabled`}
                        disabled
                      >
                        {ICON_BUTTON_ICON_MAP[variant]}
                      </IconButton>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </section>
      </div>
    </div>
  )
}

function StatCard({ title, value }: { title: string; value: string }) {
  return (
    <div className="rounded-2xl border border-border/60 bg-card/80 p-4">
      <p className="text-xs uppercase tracking-[0.2em] text-muted-foreground">
        {title}
      </p>
      <p className="mt-2 text-2xl font-semibold tracking-tight">{value}</p>
    </div>
  )
}

function getButtonLabel(variant: (typeof BUTTON_VARIANTS)[number]) {
  switch (variant) {
    case "primary":
      return "Primary Action"
    case "secondary":
      return "Secondary Action"
    case "destructive":
      return "Delete Record"
    case "outline":
      return "Outline Action"
    case "ghost":
      return "Ghost Action"
    case "link":
      return "Read Documentation"
    default:
      return "Default Action"
  }
}

function getIconLabel(variant: (typeof BUTTON_VARIANTS)[number]) {
  switch (variant) {
    case "primary":
      return "Create"
    case "secondary":
      return "Notify"
    case "destructive":
      return "Remove"
    case "outline":
      return "Export"
    case "ghost":
      return "Search"
    case "link":
      return "Learn more"
    default:
      return "Add item"
  }
}
