import type { ReactNode } from "react"
import { useTranslation } from "react-i18next"
import { Alert, Button, SearchInput } from "@workspace/ui-components"

export interface CommercePageHeaderProps {
  eyebrow: string
  title: string
  description: string
  actions?: ReactNode
}

export function CommercePageHeader({
  actions,
  description,
  eyebrow,
  title,
}: CommercePageHeaderProps) {
  return (
    <div className="flex min-w-0 flex-col gap-3 border-b border-(--app-border) bg-(--app-bg) p-4 sm:flex-row sm:items-start sm:justify-between sm:px-5">
      <div className="min-w-0">
        <p className="text-xs font-medium tracking-normal text-muted-foreground">
          {eyebrow}
        </p>
        <h1 className="mt-1 text-2xl/tight font-semibold text-foreground">
          {title}
        </h1>
        <p className="mt-1 max-w-3xl text-sm/6 text-muted-foreground">
          {description}
        </p>
      </div>
      {actions ? <div className="shrink-0">{actions}</div> : null}
    </div>
  )
}

export function CommerceSurface({ children }: { children: ReactNode }) {
  return (
    <div className="mx-auto flex max-h-full min-h-0 w-full max-w-7xl flex-1 flex-col overflow-hidden rounded-lg border border-(--app-border) bg-background">
      {children}
    </div>
  )
}

export function CommerceBody({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-0 flex-1 space-y-4 overflow-x-hidden overflow-y-auto p-4 pr-3 [scrollbar-gutter:stable] sm:p-5 sm:pr-4">
      {children}
    </div>
  )
}

export function CommercePanel({
  actions,
  children,
  description,
  title,
}: {
  actions?: ReactNode
  children: ReactNode
  description?: string
  title: string
}) {
  return (
    <section className="min-w-0 rounded-lg border border-(--app-border) bg-(--app-bg)">
      <div className="flex min-w-0 flex-col gap-2 border-b border-(--app-border) px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="min-w-0">
          <h2 className="text-base font-semibold text-foreground">{title}</h2>
          {description ? (
            <p className="mt-0.5 text-sm text-muted-foreground">
              {description}
            </p>
          ) : null}
        </div>
        {actions ? <div className="shrink-0">{actions}</div> : null}
      </div>
      <div className="p-4">{children}</div>
    </section>
  )
}

export function CommerceToolbar({
  actions,
  filters,
  keyword,
  keywordPlaceholder,
  onKeywordChange,
}: {
  actions?: ReactNode
  filters?: ReactNode
  keyword: string
  keywordPlaceholder?: string
  onKeywordChange: (value: string) => void
}) {
  const { t } = useTranslation()

  return (
    <div className="flex min-w-0 flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
      <div className="flex min-w-0 flex-1 flex-col gap-2 sm:flex-row">
        <div className="w-full sm:max-w-sm">
          <SearchInput
            value={keyword}
            placeholder={keywordPlaceholder ?? t("commerce.common.search")}
            updateStrategy="blur-enter"
            onValueChange={onKeywordChange}
          />
        </div>
        {filters}
      </div>
      {actions ? <div className="shrink-0">{actions}</div> : null}
    </div>
  )
}

export function InlineAlert({
  description,
  title,
  variant = "default",
}: {
  description: string
  title: string
  variant?: "default" | "success" | "warning" | "error"
}) {
  return <Alert variant={variant} title={title} description={description} />
}

export function SubPanelAction({
  label,
  onClick,
}: {
  label: string
  onClick: () => void
}) {
  return (
    <div className="mb-3 flex justify-end">
      <Button size="sm" variant="outline" onClick={onClick}>
        {label}
      </Button>
    </div>
  )
}
