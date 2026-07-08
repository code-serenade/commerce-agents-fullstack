import { CheckCircle2 } from "lucide-react"
import type { ReactNode } from "react"
import { useTranslation } from "react-i18next"
import { capabilityIcons, laneIcons } from "./agent-capability-icons"

function CapabilitySection({
  children,
  description,
  title,
}: {
  children: ReactNode
  description: string
  title: string
}) {
  return (
    <section className="min-w-0">
      <div className="mb-4">
        <h2 className="text-base/6 font-semibold text-foreground">{title}</h2>
        <p className="mt-1 max-w-3xl text-sm/6 text-muted-foreground">
          {description}
        </p>
      </div>
      <div className="px-4">{children}</div>
    </section>
  )
}

function CapabilityItemCard({ children }: { children: ReactNode }) {
  return (
    <div className="min-w-0 rounded-lg border border-(--app-border) bg-card/80 p-4 shadow-none">
      {children}
    </div>
  )
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

  return (
    <CapabilitySection title={t(titleKey)} description={t(descriptionKey)}>
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
    </CapabilitySection>
  )
}

export function AgentCapabilityFlowPanel({ items }: { items: string[] }) {
  const { t } = useTranslation()

  return (
    <CapabilitySection
      title={t("agentCapabilities.flow.title")}
      description={t("agentCapabilities.flow.description")}
    >
      <ol className="relative grid gap-5 before:absolute before:top-4 before:bottom-4 before:left-3.5 before:w-px before:bg-primary/30">
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
    </CapabilitySection>
  )
}

export function AgentCapabilityLanePanel({
  items,
}: {
  items: Array<{ title: string; description: string }>
}) {
  const { t } = useTranslation()

  return (
    <CapabilitySection
      title={t("agentCapabilities.lanes.title")}
      description={t("agentCapabilities.lanes.description")}
    >
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
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
    </CapabilitySection>
  )
}

export function AgentCapabilityBoundaryPanel() {
  const { t } = useTranslation()

  return (
    <CapabilitySection
      title={t("agentCapabilities.boundary.title")}
      description={t("agentCapabilities.boundary.description")}
    >
      <div className="grid gap-3 md:grid-cols-3">
        {[
          "agentCapabilities.boundary.backend",
          "agentCapabilities.boundary.context",
          "agentCapabilities.boundary.governance",
        ].map((key) => (
          <CapabilityItemCard key={key}>
            <div className="flex min-w-0 gap-3">
              <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-primary" />
              <p className="text-sm/6 text-muted-foreground">{t(key)}</p>
            </div>
          </CapabilityItemCard>
        ))}
      </div>
    </CapabilitySection>
  )
}
