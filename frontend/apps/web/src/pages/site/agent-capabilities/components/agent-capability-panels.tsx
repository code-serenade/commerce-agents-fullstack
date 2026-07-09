import {
  CheckCircle2,
  ListChecks,
  MessageCircleQuestionMark,
  Route,
  ShieldCheck,
  SlidersHorizontal,
  UsersRound,
} from "lucide-react"
import type { ReactNode } from "react"
import { useTranslation } from "react-i18next"
import {
  DocumentCard,
  DocumentSection,
} from "../../commerce/components/document-section"
import { capabilityIcons, laneIcons } from "./agent-capability-icons"

function CapabilityItemCard({ children }: { children: ReactNode }) {
  return <DocumentCard>{children}</DocumentCard>
}

export function AgentCapabilityCorePanel({
  descriptionKey = "agentCapabilities.capabilities.description",
  items,
  titleKey = "agentCapabilities.capabilities.title",
}: {
  descriptionKey?: string
  items: Array<{ title: string; description: string }>
  titleKey?: string
}) {
  const { t } = useTranslation()
  const sectionIcon =
    titleKey === "agentCapabilities.modes.title" ? (
      <SlidersHorizontal className="size-4" />
    ) : (
      <ListChecks className="size-4" />
    )

  return (
    <DocumentSection
      title={t(titleKey)}
      description={t(descriptionKey)}
      icon={sectionIcon}
    >
      <div className="grid gap-3 sm:grid-cols-2">
        {items.map((item, index) => {
          const Icon = capabilityIcons[index % capabilityIcons.length]

          return (
            <CapabilityItemCard key={item.title}>
              <div className="flex min-w-0 gap-3">
                <span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
                  <Icon className="size-4" />
                </span>
                <div className="min-w-0">
                  <h3 className="min-w-0 text-sm/5 font-semibold text-foreground">
                    {item.title}
                  </h3>
                  <p className="mt-1 text-sm/6 text-muted-foreground">
                    {item.description}
                  </p>
                </div>
              </div>
            </CapabilityItemCard>
          )
        })}
      </div>
    </DocumentSection>
  )
}

export function AgentCapabilityFlowPanel({ items }: { items: string[] }) {
  const { t } = useTranslation()

  return (
    <DocumentSection
      title={t("agentCapabilities.flow.title")}
      description={t("agentCapabilities.flow.description")}
      icon={<Route className="size-4" />}
    >
      <ol className="relative grid gap-5 before:absolute before:inset-y-4 before:left-3.5 before:w-px before:bg-primary/30">
        {items.map((item, index) => (
          <li key={item} className="relative flex min-w-0 items-start gap-4">
            <span className="z-10 flex size-7 shrink-0 items-center justify-center rounded-full border border-primary/30 bg-background text-[11px] font-semibold text-primary shadow-none ring-4 ring-primary/10">
              {String(index + 1).padStart(2, "0")}
            </span>
            <span className="pt-0.5 text-sm/6 text-muted-foreground">
              {item}
            </span>
          </li>
        ))}
      </ol>
    </DocumentSection>
  )
}

export function AgentCapabilityQaPanel({
  items,
}: {
  items: Array<{ title: string; description: string }>
}) {
  const { t } = useTranslation()

  return (
    <DocumentSection
      title={t("agentCapabilities.qa.title")}
      description={t("agentCapabilities.qa.description")}
      icon={<MessageCircleQuestionMark className="size-4" />}
    >
      <ul className="divide-y divide-(--app-border)/70">
        {items.map((item) => (
          <li key={item.title} className="min-w-0 py-3 first:pt-0 last:pb-0">
            <h3 className="text-sm/5 font-medium text-foreground">
              {item.title}
            </h3>
            <p className="mt-1.5 text-sm/6 text-muted-foreground">
              {item.description}
            </p>
          </li>
        ))}
      </ul>
    </DocumentSection>
  )
}

export function AgentCapabilityLanePanel({
  items,
}: {
  items: Array<{ title: string; description: string }>
}) {
  const { t } = useTranslation()

  return (
    <DocumentSection
      title={t("agentCapabilities.lanes.title")}
      description={t("agentCapabilities.lanes.description")}
      icon={<UsersRound className="size-4" />}
    >
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        {items.map((item, index) => {
          const Icon = laneIcons[index % laneIcons.length]

          return (
            <CapabilityItemCard key={item.title}>
              <div className="flex min-w-0 gap-3">
                <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
                  <Icon className="size-4" />
                </span>
                <div className="min-w-0">
                  <h3 className="text-sm font-semibold text-foreground">
                    {item.title}
                  </h3>
                  <p className="mt-1 text-xs/5 text-muted-foreground">
                    {item.description}
                  </p>
                </div>
              </div>
            </CapabilityItemCard>
          )
        })}
      </div>
    </DocumentSection>
  )
}

export function AgentCapabilityBoundaryPanel() {
  const { t } = useTranslation()

  return (
    <DocumentSection
      title={t("agentCapabilities.boundary.title")}
      description={t("agentCapabilities.boundary.description")}
      icon={<ShieldCheck className="size-4" />}
    >
      <ul className="divide-y divide-(--app-border)/70">
        {[
          "agentCapabilities.boundary.backend",
          "agentCapabilities.boundary.context",
          "agentCapabilities.boundary.governance",
        ].map((key) => (
          <li
            key={key}
            className="flex min-w-0 gap-3 py-3 first:pt-0 last:pb-0"
          >
            <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-primary" />
            <p className="text-sm/6 text-muted-foreground">{t(key)}</p>
          </li>
        ))}
      </ul>
    </DocumentSection>
  )
}
