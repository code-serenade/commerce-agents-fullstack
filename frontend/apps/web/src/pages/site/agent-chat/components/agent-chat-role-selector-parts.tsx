import { useTranslation } from "react-i18next"
import type { AgentRuntimeRoleOption } from "../domain/agent-runtime-options"

export function RoleChoiceButton({
  disabled,
  onClick,
  role,
  selected,
  variant = "default",
}: {
  disabled: boolean
  onClick: (value: string) => void
  role: AgentRuntimeRoleOption
  selected: boolean
  variant?: "default" | "lead" | "support"
}) {
  const { t } = useTranslation()
  const badge =
    variant === "lead" && selected
      ? t("agentChat.composer.runtime.primaryBadge")
      : variant === "support" && selected
        ? t("agentChat.composer.runtime.supportBadge")
        : ""

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={() => onClick(role.runtimeRole)}
      aria-label={role.name}
      aria-pressed={selected}
      data-runtime-role={role.runtimeRole}
      data-runtime-role-kind={variant}
      className={`group flex h-9 min-w-0 items-center justify-center gap-1.5 rounded-md border px-2 text-center transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
        selected
          ? "border-primary/60 bg-primary/10 text-primary ring-1 ring-primary/15"
          : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground"
      }`}
      title={role.capabilities.join(" / ")}
    >
      <span className="min-w-0 truncate text-sm font-semibold">
        {role.name}
      </span>
      {badge ? (
        <span className="shrink-0 rounded-full bg-primary px-1.5 py-0.5 text-[10px] text-primary-foreground">
          {badge}
        </span>
      ) : null}
    </button>
  )
}

export function RoleAssignmentRow({
  disabled,
  isPrimary,
  isSupport,
  onPrimaryRoleChange,
  onToggleRole,
  role,
}: {
  disabled: boolean
  isPrimary: boolean
  isSupport: boolean
  onPrimaryRoleChange: (value: string) => void
  onToggleRole: (value: string) => void
  role: AgentRuntimeRoleOption
}) {
  const { t } = useTranslation()

  return (
    <div
      className={`grid min-w-0 grid-cols-[minmax(0,1fr)_4.75rem_4.75rem] items-center gap-1.5 rounded-md border px-2 py-1.5 ${
        isPrimary || isSupport
          ? "border-primary/25 bg-primary/5"
          : "border-border bg-card"
      }`}
    >
      <button
        type="button"
        disabled={disabled}
        onClick={() => onPrimaryRoleChange(role.runtimeRole)}
        className="min-w-0 text-left disabled:cursor-not-allowed disabled:opacity-50"
        title={role.capabilities.join(" / ")}
      >
        <span className="block truncate text-sm font-semibold text-foreground">
          {role.name}
        </span>
        <span className="block truncate text-xs text-muted-foreground">
          {role.capabilities.slice(0, 2).join(" / ")}
        </span>
      </button>
      <button
        type="button"
        disabled={disabled}
        onClick={() => onPrimaryRoleChange(role.runtimeRole)}
        aria-pressed={isPrimary}
        className={`h-8 rounded-md border px-2 text-xs font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
          isPrimary
            ? "border-primary/60 bg-primary text-primary-foreground"
            : "border-border bg-background text-muted-foreground hover:bg-muted hover:text-foreground"
        }`}
      >
        {t("agentChat.composer.runtime.primaryShort")}
      </button>
      <button
        type="button"
        disabled={disabled || isPrimary}
        onClick={() => onToggleRole(role.runtimeRole)}
        aria-pressed={isSupport}
        className={`h-8 rounded-md border px-2 text-xs font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
          isSupport
            ? "border-primary/60 bg-primary/10 text-primary"
            : "border-border bg-background text-muted-foreground hover:bg-muted hover:text-foreground"
        }`}
      >
        {t("agentChat.composer.runtime.supportShort")}
      </button>
    </div>
  )
}
