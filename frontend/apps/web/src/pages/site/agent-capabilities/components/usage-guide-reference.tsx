import {
  BellRing,
  Bot,
  Boxes,
  CheckCircle2,
  ClipboardCheck,
  ArrowUpRight,
  FileText,
  Library,
  Megaphone,
  ShieldAlert,
  ShoppingBag,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import { Link } from "react-router"
import { DocumentSection } from "../../commerce/components/document-section"
import { getUsageGuideContent } from "../usage-guide-data"

const featureIcons = [
  Bot,
  ShoppingBag,
  Library,
  Megaphone,
  FileText,
  ClipboardCheck,
  ShieldAlert,
  BellRing,
]

const featurePaths = [
  "/commerce",
  "/products",
  "/materials",
  "/campaigns",
  "/agent-output",
  "/workspaces",
  "/approvals",
  "/operations",
] as const

export function UsageGuideReference() {
  const { t } = useTranslation()
  const content = getUsageGuideContent(t)

  return (
    <>
      <DocumentSection
        title={content.reference.title}
        description={content.reference.description}
        icon={<Boxes className="size-4" />}
        contentClassName="pl-0 sm:pl-9"
      >
        <div className="grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
          {content.reference.items.map((item, index) => {
            const Icon = featureIcons[index] ?? Bot
            return (
              <Link
                key={item.title}
                to={featurePaths[index] ?? "/commerce"}
                className="group relative flex min-h-24 min-w-0 cursor-pointer items-center gap-3 rounded-lg border border-(--app-border) bg-background px-4 py-3 pr-12 text-left no-underline transition-colors duration-200 hover:border-(--app-border) hover:bg-muted/25 focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-primary/20 focus-visible:outline-none"
              >
                <span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-primary/8 text-primary">
                  <Icon className="size-4" />
                </span>
                <div className="min-w-0 self-center">
                  <h3 className="text-sm font-semibold text-foreground">
                    {item.title}
                  </h3>
                  <p className="mt-1 text-xs/5 text-muted-foreground">
                    {item.detail}
                  </p>
                </div>
                <ArrowUpRight className="absolute top-1/2 right-4 size-4 -translate-y-1/2 text-muted-foreground opacity-50 transition-[transform,color,opacity] group-hover:translate-x-0.5 group-hover:translate-y-[calc(-50%-0.125rem)] group-hover:text-primary group-hover:opacity-100" />
              </Link>
            )
          })}
        </div>
      </DocumentSection>

      <DocumentSection
        title={content.boundary.title}
        description={content.boundary.description}
        icon={<ShieldAlert className="size-4" />}
        contentClassName="pl-0 sm:pl-9"
      >
        <div className="grid gap-3 lg:grid-cols-2">
          <div className="min-w-0 rounded-lg border border-(--app-border) bg-background p-4">
            <h3 className="text-sm font-semibold text-foreground">
              {content.boundary.supportedTitle}
            </h3>
            <ul className="mt-3 space-y-2">
              {content.boundary.supported.map((item) => (
                <li key={item} className="flex min-w-0 gap-2 text-sm/6 text-muted-foreground">
                  <CheckCircle2 className="mt-1 size-4 shrink-0 text-emerald-600" />
                  <span>{item}</span>
                </li>
              ))}
            </ul>
          </div>
          <div className="min-w-0 rounded-lg border border-(--app-border) bg-background p-4">
            <h3 className="text-sm font-semibold text-foreground">
              {content.boundary.guardedTitle}
            </h3>
            <ul className="mt-3 space-y-2">
              {content.boundary.guarded.map((item) => (
                <li key={item} className="flex min-w-0 gap-2 text-sm/6 text-muted-foreground">
                  <ShieldAlert className="mt-1 size-4 shrink-0 text-destructive" />
                  <span>{item}</span>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </DocumentSection>
    </>
  )
}
