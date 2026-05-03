import type { ComponentType, ReactNode } from "react"
import { Card, CardContent } from "@workspace/ui-core/components/card"

export type MetricCardsVariant =
  | "default"
  | "accent"
  | "success"
  | "warning"
  | "danger"

export type MetricCardsLayout = "grid" | "inline"

export interface MetricCardsItem {
  key?: string
  label: ReactNode
  value: ReactNode
  description?: ReactNode
  tail?: ReactNode
  icon?: ReactNode
  variant?: MetricCardsVariant
}

export interface MetricCardsProps {
  items: readonly MetricCardsItem[]
  variant?: MetricCardsVariant
  layout?: MetricCardsLayout
  card?: ComponentType<MetricCardsCardProps>
}

export interface MetricCardsCardClassNames {
  card: string
  iconWrap: string
  icon: string
  label: string
  value: string
  tail: string
}

export interface MetricCardsCardProps {
  item: MetricCardsItem
  index: number
  layout: MetricCardsLayout
  variant: MetricCardsVariant
  classNames: MetricCardsCardClassNames
}

const metricCardVariantClassNames: Record<
  MetricCardsVariant,
  MetricCardsCardClassNames
> = {
  default: {
    card: "bg-[var(--surface)] ring-0 shadow-none",
    iconWrap: "bg-[color:color-mix(in_oklab,var(--foreground)_8%,var(--surface))]",
    icon: "text-[var(--foreground)]",
    label: "text-[var(--muted-foreground)]",
    value: "text-[var(--foreground)]",
    tail: "text-[var(--muted-foreground)]",
  },
  accent: {
    card: "bg-[var(--accent)] ring-0 shadow-none",
    iconWrap: "bg-[color:color-mix(in_oklab,var(--foreground)_8%,var(--accent))]",
    icon: "text-[var(--foreground)]",
    label: "text-[var(--muted-foreground)]",
    value: "text-[var(--foreground)]",
    tail: "text-[var(--muted-foreground)]",
  },
  success: {
    card:
      "bg-[color:color-mix(in_oklab,var(--success)_10%,var(--surface))] ring-0 shadow-none",
    iconWrap:
      "bg-[color:color-mix(in_oklab,var(--success)_18%,var(--surface))]",
    icon: "text-[var(--success)]",
    label: "text-[var(--muted-foreground)]",
    value: "text-[var(--success)]",
    tail: "text-[var(--foreground)]",
  },
  warning: {
    card:
      "bg-[color:color-mix(in_oklab,var(--warning,#d97706)_10%,var(--surface))] ring-0 shadow-none",
    iconWrap:
      "bg-[color:color-mix(in_oklab,var(--warning,#d97706)_18%,var(--surface))]",
    icon: "text-[var(--warning,#b45309)]",
    label: "text-[var(--muted-foreground)]",
    value: "text-[var(--warning,#b45309)]",
    tail: "text-[var(--foreground)]",
  },
  danger: {
    card:
      "bg-[color:color-mix(in_oklab,var(--destructive)_10%,var(--surface))] ring-0 shadow-none",
    iconWrap:
      "bg-[color:color-mix(in_oklab,var(--destructive)_18%,var(--surface))]",
    icon: "text-[var(--destructive)]",
    label: "text-[var(--muted-foreground)]",
    value: "text-[var(--destructive)]",
    tail: "text-[var(--foreground)]",
  },
}

function DefaultMetricCard({
  item,
  layout,
  classNames,
}: MetricCardsCardProps) {
  if (layout === "inline") {
    return (
      <Card className={`min-w-34 ${classNames.card}`}>
        <CardContent className="px-4 py-0">
          <div className="flex min-h-10 items-center gap-2.5">
            {item.icon ? (
              <span className={`flex size-4 shrink-0 items-center justify-center ${classNames.icon}`}>
                {item.icon}
              </span>
            ) : null}

            <div className="min-w-0">
              <p className={`truncate text-sm font-medium ${classNames.label}`}>
                {item.label}
              </p>
              <p
                className={`mt-0.5 truncate text-xl leading-none font-semibold ${classNames.value}`}
              >
                {item.value}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className={`h-full min-w-0 ${classNames.card}`}>
      <CardContent className="px-3 py-2 sm:px-3.5 sm:py-2.5">
        <div className="flex min-h-14 items-center gap-2.5 sm:min-h-16 sm:gap-3">
          {item.icon ? (
            <div
              className={`hidden size-8 shrink-0 items-center justify-center rounded-lg sm:flex sm:size-9 ${classNames.iconWrap}`}
            >
              <span className={`flex items-center justify-center ${classNames.icon}`}>
                {item.icon}
              </span>
            </div>
          ) : null}

          <div className="min-w-0 flex-1 text-left sm:text-left">
            <p className={`truncate text-xs sm:text-sm ${classNames.label}`}>
              {item.label}
            </p>
            <p
              className={`mt-0.5 truncate text-xl leading-none font-semibold sm:text-2xl ${classNames.value}`}
            >
              {item.value}
            </p>
            <p
              className={`mt-0.5 line-clamp-2 text-[11px] leading-3.5 sm:text-xs sm:leading-4 ${classNames.tail}`}
            >
              {item.tail ?? item.description}
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

export function MetricCards({
  items,
  layout = "grid",
  variant = "default",
  card: CardComponent = DefaultMetricCard,
}: MetricCardsProps) {
  const rootClassName =
    layout === "inline"
      ? "inline-flex w-fit max-w-full flex-wrap gap-2 sm:gap-3"
      : "grid w-full min-w-0 grid-cols-4 gap-2 sm:gap-3 xl:gap-4"

  return (
    <div className={rootClassName}>
      {items.map((item, index) => {
        const resolvedVariant = item.variant ?? variant
        const classNames = metricCardVariantClassNames[resolvedVariant]

        return (
          <CardComponent
            key={item.key ?? (typeof item.label === "string" ? item.label : index)}
            item={item}
            index={index}
            layout={layout}
            variant={resolvedVariant}
            classNames={classNames}
          />
        )
      })}
    </div>
  )
}
