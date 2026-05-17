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
    <div className="flex min-w-0 flex-wrap items-center gap-1.5 rounded-md border border-border bg-background px-2.5 py-1.5 text-xs text-muted-foreground">
      {primaryRole ? (
        <span className="inline-flex min-w-0 items-center gap-1.5">
          <Crown className="size-3 shrink-0 text-primary" />
          <span className="shrink-0">
            {t("agentChat.composer.runtime.currentLead")}
          </span>
          <span className="max-w-32 truncate font-semibold text-foreground">
            {primaryRole.name}
          </span>
        </span>
      ) : null}
      <span className="text-border">/</span>
      <span className="min-w-0">
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
    <div className="flex min-w-0 items-start gap-2">
      <span className="mt-0.5 text-muted-foreground">{icon}</span>
      <span className="min-w-0">
        <span className="block text-xs font-semibold text-foreground">
          {title}
        </span>
        <span className="wrap-break-word mt-0.5 block text-xs/5 text-muted-foreground">
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
      className={`flex min-w-0 items-start gap-2 rounded-lg border px-3 py-2 text-left transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
        selected
          ? "border-primary/70 bg-background text-primary shadow-sm ring-2 ring-primary/20"
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
        <span className="flex min-w-0 items-center gap-1.5 text-sm font-semibold">
          <span className="truncate">{role.name}</span>
          {selected ? (
            <span className="shrink-0 rounded-full bg-primary px-1.5 py-0.5 text-[10px] text-primary-foreground">
              {badge}
            </span>
          ) : null}
        </span>
        {capability ? (
          <span className="mt-1 line-clamp-2 block text-xs/5 opacity-80">
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
      className={`flex max-w-52 min-w-0 items-center gap-1.5 rounded-md border px-2.5 py-1.5 text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
        selected
          ? "border-primary/50 bg-primary/10 text-primary"
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
