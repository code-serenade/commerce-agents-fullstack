import { CheckCircle2 } from "lucide-react"
import { useTranslation } from "react-i18next"
import { CommercePanel } from "../../commerce/components/commerce-ui"
import { capabilityIcons, laneIcons } from "./agent-capability-icons"

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
    <CommercePanel
      title={t(titleKey)}
      description={t(descriptionKey)}
    >
      <div className="grid gap-3 sm:grid-cols-2">
        {items.map((item, index) => {
          const Icon = capabilityIcons[index % capabilityIcons.length]

          return (
            <div
              key={item.title}
              className="min-w-0 rounded-md border border-(--app-border) bg-background p-3"
            >
              <div className="flex min-w-0 items-center gap-2">
                <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted text-foreground">
                  <Icon className="size-4" />
                </span>
                <h2 className="min-w-0 text-sm/5 font-semibold text-foreground">
                  {item.title}
                </h2>
              </div>
              <p className="mt-2 text-sm/6 text-muted-foreground">
                {item.description}
              </p>
            </div>
          )
        })}
      </div>
    </CommercePanel>
  )
}

export function AgentCapabilityFlowPanel({ items }: { items: string[] }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("agentCapabilities.flow.title")}
      description={t("agentCapabilities.flow.description")}
    >
      <ol className="space-y-3">
        {items.map((item, index) => (
          <li key={item} className="flex gap-3">
            <span className="flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-xs font-semibold text-foreground">
              {index + 1}
            </span>
            <span className="text-sm/6 text-muted-foreground">{item}</span>
          </li>
        ))}
      </ol>
    </CommercePanel>
  )
}

export function AgentCapabilityLanePanel({
  items,
}: {
  items: Array<{ title: string; description: string }>
}) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("agentCapabilities.lanes.title")}
      description={t("agentCapabilities.lanes.description")}
    >
      <div className="grid gap-3 md:grid-cols-5">
        {items.map((item, index) => {
          const Icon = laneIcons[index % laneIcons.length]

          return (
            <div key={item.title} className="min-w-0 rounded-md bg-muted/60 p-3">
              <Icon className="size-4 text-foreground" />
              <h2 className="mt-2 text-sm font-semibold text-foreground">
                {item.title}
              </h2>
              <p className="mt-1 text-xs/5 text-muted-foreground">
                {item.description}
              </p>
            </div>
          )
        })}
      </div>
    </CommercePanel>
  )
}

export function AgentCapabilityBoundaryPanel() {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("agentCapabilities.boundary.title")}
      description={t("agentCapabilities.boundary.description")}
    >
      <div className="grid gap-3 md:grid-cols-3">
        {[
          "agentCapabilities.boundary.backend",
          "agentCapabilities.boundary.context",
          "agentCapabilities.boundary.governance",
        ].map((key) => (
          <div
            key={key}
            className="flex min-w-0 gap-3 rounded-md border border-(--app-border) bg-background p-3"
          >
            <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-emerald-600" />
            <p className="text-sm/6 text-muted-foreground">{t(key)}</p>
          </div>
        ))}
      </div>
    </CommercePanel>
  )
}
