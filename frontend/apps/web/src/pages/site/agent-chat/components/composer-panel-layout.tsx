import { X } from "lucide-react"
import { createContext, useContext, type ReactNode } from "react"
import { Button } from "@workspace/ui-components"

interface ComposerPanelCloseContextValue {
  label: string
  onClose: () => void
}

const ComposerPanelCloseContext =
  createContext<ComposerPanelCloseContextValue | null>(null)

function cx(...inputs: Array<string | false | null | undefined>) {
  return inputs.filter(Boolean).join(" ")
}

export function ComposerPanelCloseProvider({
  children,
  label,
  onClose,
}: ComposerPanelCloseContextValue & {
  children: ReactNode
}) {
  return (
    <ComposerPanelCloseContext.Provider value={{ label, onClose }}>
      {children}
    </ComposerPanelCloseContext.Provider>
  )
}

export const composerPanelActionButtonClassName =
  "inline-flex h-8 shrink-0 items-center gap-1.5 rounded-lg bg-foreground px-3 text-xs font-medium text-background transition-colors hover:bg-foreground/90 disabled:cursor-not-allowed disabled:opacity-50"

export const composerPanelSecondaryButtonClassName =
  "inline-flex h-8 shrink-0 items-center gap-1.5 rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-3 text-xs font-medium text-muted-foreground transition-colors hover:bg-(--agent-chat-accent-soft) hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"

export const composerPanelTextButtonClassName =
  "inline-flex h-8 shrink-0 items-center gap-1.5 rounded-lg bg-transparent px-1 text-xs font-medium text-primary transition-colors hover:text-primary/80 disabled:cursor-not-allowed disabled:opacity-50"

export const composerPanelTinyButtonClassName =
  "inline-flex h-7 shrink-0 items-center gap-1.5 rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-2.5 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-(--agent-chat-accent-soft) hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"

export const composerPanelListClassName =
  "grid min-w-0 grid-cols-[repeat(auto-fit,minmax(min(17rem,100%),1fr))] gap-2"

export const composerPanelSelectedAreaClassName =
  "grid grid-cols-[repeat(auto-fit,minmax(min(17rem,100%),1fr))] gap-2 rounded-xl border border-(--agent-chat-soft-border) bg-(--agent-chat-subtle-surface) p-2.5"

export function ComposerPanelHeader({
  actions,
  description,
  title,
}: {
  actions?: ReactNode
  description: ReactNode
  title: ReactNode
}) {
  const closeAction = useContext(ComposerPanelCloseContext)

  return (
    <div className="relative mb-4 flex min-w-0 flex-wrap items-center justify-between gap-3 pr-10 pb-3 after:absolute after:bottom-0 after:left-0 after:h-px after:w-full after:bg-[linear-gradient(90deg,var(--agent-chat-soft-border),transparent)]">
      <div className="min-w-0">
        <p className="text-sm/5 font-bold text-foreground">{title}</p>
        <p className="mt-0.5 text-[11px]/4 text-muted-foreground">
          {description}
        </p>
      </div>
      {actions ? (
        <div className="flex shrink-0 flex-wrap items-center gap-1.5">
          {actions}
        </div>
      ) : null}
      {closeAction ? (
        <Button
          type="button"
          aria-label={closeAction.label}
          title={closeAction.label}
          onClick={closeAction.onClose}
          classNameMode="replace"
          className="absolute top-0 right-0 inline-flex size-8 shrink-0 items-center justify-center rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) text-muted-foreground shadow-sm transition-colors hover:bg-(--agent-chat-accent-soft) hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-hidden"
        >
          <X className="size-4" aria-hidden="true" />
        </Button>
      ) : null}
    </div>
  )
}

export function ComposerPanelFrame({
  actions,
  children,
  description,
  title,
}: {
  actions?: ReactNode
  children: ReactNode
  description: ReactNode
  title: ReactNode
}) {
  return (
    <div className="flex h-full max-h-full min-h-0 flex-1 flex-col overflow-hidden">
      <ComposerPanelHeader
        actions={actions}
        description={description}
        title={title}
      />
      <div
        className="agent-chat-scrollarea min-h-0 flex-1 overflow-x-hidden overflow-y-auto overscroll-contain pr-3 pb-9 [scrollbar-gutter:stable]"
        data-agent-chat-composer-panel-scroll="true"
      >
        {children}
      </div>
    </div>
  )
}

export function ComposerPanelSection({
  aside,
  children,
  className,
  description,
  title,
}: {
  aside?: ReactNode
  children?: ReactNode
  className?: string
  description?: ReactNode
  title: ReactNode
}) {
  return (
    <section className={cx("grid min-w-0 gap-2", className)}>
      <div className="flex min-w-0 flex-wrap items-end justify-between gap-2">
        <div className="min-w-0">
          <p className="text-[11px] font-semibold text-foreground">{title}</p>
          {description ? (
            <p className="mt-0.5 text-[11px] text-muted-foreground">
              {description}
            </p>
          ) : null}
        </div>
        {aside ? (
          <div className="flex shrink-0 flex-wrap items-center gap-1.5">
            {aside}
          </div>
        ) : null}
      </div>
      {children}
    </section>
  )
}

export function ComposerPanelEmptyState({
  action,
  children,
  icon,
}: {
  action?: ReactNode
  children: ReactNode
  icon?: ReactNode
}) {
  return (
    <div className="col-span-full flex min-w-0 flex-wrap items-center justify-between gap-3 rounded-xl border border-dashed border-(--agent-chat-soft-border) bg-(--agent-chat-subtle-surface) p-3 text-xs text-muted-foreground">
      <div className="flex min-w-0 items-start gap-2">
        {icon ? (
          <span className="mt-0.5 flex size-4 shrink-0 items-center justify-center text-muted-foreground">
            {icon}
          </span>
        ) : null}
        <span className="min-w-0">{children}</span>
      </div>
      {action}
    </div>
  )
}
