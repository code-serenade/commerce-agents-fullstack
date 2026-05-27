import type { ReactNode } from "react"
import { CopyableText } from "@workspace/app-kit"

export function FilesDetailSection({
  children,
  title,
}: {
  children: ReactNode
  title: string
}) {
  return (
    <section className="grid gap-2">
      <h3 className="text-sm font-medium text-foreground">{title}</h3>
      <dl className="grid gap-2 rounded-lg border border-(--app-border) bg-muted/30 p-3">
        {children}
      </dl>
    </section>
  )
}

export function FilesDetailRow({
  copyValue,
  label,
  value,
}: {
  copyValue?: string | null
  label: string
  value: ReactNode
}) {
  const displayValue =
    value === null || value === undefined || value === "" ? "-" : value
  const canCopy = typeof copyValue === "string" && copyValue.trim().length > 0

  return (
    <div className="grid gap-1 sm:grid-cols-[9rem_minmax(0,1fr)]">
      <dt className="text-xs text-muted-foreground">{label}</dt>
      <dd className="min-w-0 break-all text-xs text-foreground">
        {canCopy ? (
          <CopyableText
            className="min-w-0"
            textClassName="whitespace-normal break-all"
            value={copyValue}
          >
            {displayValue}
          </CopyableText>
        ) : (
          displayValue
        )}
      </dd>
    </div>
  )
}
