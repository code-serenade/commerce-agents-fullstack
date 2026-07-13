import type { ReactNode } from "react"
import { useTranslation } from "react-i18next"
import {
  Alert,
  Button,
  type ButtonProps,
  SearchInput,
} from "@workspace/ui-components"

export interface CommercePageHeaderProps {
  className?: string
  eyebrow: string
  title: string
  description: string
  actions?: ReactNode
}

function cx(...inputs: Array<string | false | null | undefined>) {
  return inputs.filter(Boolean).join(" ")
}

export function CommercePageHeader({
  actions,
  className,
}: CommercePageHeaderProps) {
  if (!actions) return null

  return (
    <div
      className={cx(
        "flex justify-end p-4 pb-0 [&_a]:h-10 [&_a]:border-primary [&_a]:bg-primary [&_a]:text-primary-foreground dark:[&_a]:border-[var(--primary-outline,var(--primary))] dark:[&_a]:bg-[var(--surface)] dark:[&_a]:text-[var(--primary-outline,var(--primary))] [&_button]:h-10 [&_button]:border-primary [&_button]:bg-primary [&_button]:text-primary-foreground dark:[&_button]:border-[var(--primary-outline,var(--primary))] dark:[&_button]:bg-[var(--surface)] dark:[&_button]:text-[var(--primary-outline,var(--primary))]",
        className
      )}
    >
      {actions}
    </div>
  )
}

export function CommerceSurface({ children }: { children: ReactNode }) {
  return (
    <div className="flex max-h-full min-h-0 w-full flex-1 flex-col overflow-hidden bg-transparent">
      {children}
    </div>
  )
}

export function CommerceBody({
  children,
  className,
}: {
  children: ReactNode
  className?: string
}) {
  return (
    <div
      className={cx(
        "min-h-0 flex-1 space-y-4 overflow-x-hidden overflow-y-auto p-4 [scrollbar-gutter:stable]",
        className
      )}
    >
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
  title?: string
}) {
  const hasHeader = Boolean(title || description || actions)

  return (
    <section className="min-w-0 rounded-lg border border-(--app-border) bg-(--app-bg)">
      {hasHeader ? (
        <div className="flex min-w-0 flex-col gap-2 border-b border-(--app-border) px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
          <div className="min-w-0">
            {title ? (
              <h2 className="text-base font-semibold text-foreground">
                {title}
              </h2>
            ) : null}
            {description ? (
              <p className="mt-0.5 text-sm text-muted-foreground">
                {description}
              </p>
            ) : null}
          </div>
          {actions ? <div className="shrink-0">{actions}</div> : null}
        </div>
      ) : null}
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
  buttonSize = "sm",
  label,
  onClick,
}: {
  buttonSize?: ButtonProps["size"]
  label: string
  onClick: () => void
}) {
  return (
    <div className="mb-3 flex justify-end">
      <Button size={buttonSize} variant="outline" onClick={onClick}>
        {label}
      </Button>
    </div>
  )
}
