import type { ReactNode } from "react"
import { Check, Crown } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeRoleOption } from "../domain/agent-runtime-options"

export function SelectedRoleSummary({
  primaryRole,
  supportRoles,
}: {
  primaryRole?: AgentRuntimeRoleOption
  supportRoles: AgentRuntimeRoleOption[]
}) {
  const { t } = useTranslation()

  return (
    <div className="grid min-w-0 gap-1.5 rounded-lg border border-primary/20 bg-background px-3 py-2 text-xs text-muted-foreground sm:grid-cols-[minmax(0,0.85fr)_minmax(0,1.15fr)] sm:items-center">
      {primaryRole ? (
        <span className="inline-flex min-w-0 items-center gap-1.5 rounded-md bg-primary/10 px-2 py-1 text-primary">
          <Crown className="size-3 shrink-0 text-primary" />
          <span className="shrink-0">
            {t("agentChat.composer.runtime.currentLead")}
          </span>
          <span className="truncate font-semibold text-primary">
            {primaryRole.name}
          </span>
        </span>
      ) : null}
      <span className="min-w-0 truncate">
        {supportRoles.length
          ? t("agentChat.composer.runtime.currentSupport", {
              roles: supportRoles.map((role) => role.name).join("、"),
            })
          : t("agentChat.composer.runtime.noSupportRoles")}
      </span>
    </div>
  )
}

export function RoleSectionHeader({
  description,
  icon,
  title,
}: {
  description: string
  icon: ReactNode
  title: string
}) {
  return (
    <div className="flex min-w-0 items-center gap-2" title={description}>
      <span className="text-muted-foreground">{icon}</span>
      <span className="flex min-w-0 items-baseline gap-2">
        <span className="shrink-0 text-xs font-semibold text-foreground">
          {title}
        </span>
        <span className="truncate text-xs text-muted-foreground">
          {description}
        </span>
      </span>
    </div>
  )
}

export function PrimaryRoleButton({
  badge,
  disabled,
  onSelect,
  role,
  selected,
}: {
  badge: string
  disabled: boolean
  onSelect: (value: string) => void
  role: AgentRuntimeRoleOption
  selected: boolean
}) {
  const capability = role.capabilities.slice(0, 2).join(" / ")

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={() => onSelect(role.runtimeRole)}
      aria-label={`${badge}：${role.name}`}
      aria-pressed={selected}
      data-runtime-role={role.runtimeRole}
      data-runtime-role-kind="primary"
      className={`flex min-w-0 items-start gap-2 rounded-lg border px-3 py-2 text-left transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
        selected
          ? "border-primary bg-primary/10 text-primary shadow-sm ring-2 ring-primary/25"
          : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground"
      }`}
      title={role.capabilities.join(" / ")}
    >
      <span
        className={`mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full border ${
          selected ? "border-primary bg-primary text-primary-foreground" : ""
        }`}
      >
        {selected ? <Check className="size-3" /> : null}
      </span>
      <span className="min-w-0">
        <span className="flex min-w-0 flex-wrap items-center gap-1.5 text-sm font-semibold">
          <span className="truncate">{role.name}</span>
          {selected ? (
            <span className="shrink-0 rounded-full bg-primary px-2 py-0.5 text-[10px] text-primary-foreground">
              {badge}
            </span>
          ) : null}
        </span>
        {capability ? (
          <span className="mt-1 line-clamp-1 block text-xs/5 opacity-80">
            {capability}
          </span>
        ) : null}
      </span>
    </button>
  )
}

export function RoleToggle({
  disabled,
  onToggleRole,
  role,
  selected,
}: {
  disabled: boolean
  onToggleRole: (value: string) => void
  role: AgentRuntimeRoleOption
  selected: boolean
}) {
  const { t } = useTranslation()

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={() => onToggleRole(role.runtimeRole)}
      aria-label={`${t("agentChat.composer.runtime.supportBadge")}：${role.name}`}
      aria-pressed={selected}
      data-runtime-role={role.runtimeRole}
      data-runtime-role-kind="support"
      className={`flex min-w-0 items-center gap-1.5 rounded-md border px-2.5 py-1.5 text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
        selected
          ? "border-primary/60 bg-primary/10 text-primary ring-1 ring-primary/15"
          : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground"
      }`}
      title={role.capabilities.join(" / ")}
    >
      {selected ? <Check className="size-3.5 shrink-0" /> : null}
      <span className="truncate">{role.name}</span>
      {selected ? (
        <span className="shrink-0 rounded-full bg-current/10 px-1.5 py-0.5 text-[10px]">
          {t("agentChat.composer.runtime.supportBadge")}
        </span>
      ) : null}
    </button>
  )
}
