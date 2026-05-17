import { Check, UsersRound } from "lucide-react"
import { useTranslation } from "react-i18next"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeRoleOption,
} from "../domain/agent-runtime-options"
import { roleToSelectOption } from "../view-model/agent-chat-runtime-options-view-model"
import { RuntimeSelect } from "./agent-chat-runtime-shared"

export function RoleSelector({
  disabled,
  onPrimaryRoleChange,
  onToggleRole,
  roles,
  runtimeSelection,
}: {
  disabled: boolean
  onPrimaryRoleChange: (value: string) => void
  onToggleRole: (value: string) => void
  roles: AgentRuntimeRoleOption[]
  runtimeSelection: AgentChatRuntimeSelection
}) {
  const { t } = useTranslation()
  const roleOptions = roles.map(roleToSelectOption)
  const isAuto = runtimeSelection.collaborationMode === "auto"
  const isSingle = runtimeSelection.collaborationMode === "single"

  if (isAuto) {
    return (
      <div className="flex min-w-0 items-center gap-2 rounded-lg border border-border bg-card px-3 py-2 text-xs text-muted-foreground">
        <UsersRound className="size-3.5 shrink-0" />
        <span className="wrap-break-word">
          {t("agentChat.composer.runtime.autoRolesHint")}
        </span>
      </div>
    )
  }

  return (
    <div className="grid gap-2">
      <div className="flex min-w-0 flex-wrap items-center gap-2">
        <span className="shrink-0 text-xs font-semibold text-foreground">
          {isSingle
            ? t("agentChat.composer.runtime.singleRole")
            : t("agentChat.composer.runtime.roles")}
        </span>
        <RuntimeSelect
          disabled={disabled}
          label={t("agentChat.composer.runtime.primary")}
          onValueChange={onPrimaryRoleChange}
          options={roleOptions}
          value={runtimeSelection.primaryRole ?? ""}
        />
      </div>
      {!isSingle ? (
        <>
          <p className="text-xs/5 text-muted-foreground">
            {t("agentChat.composer.runtime.manualRolesHint")}
          </p>
          <div className="flex min-w-0 flex-wrap gap-1.5">
            {roles.map((role) => (
              <RoleToggle
                disabled={disabled}
                key={role.runtimeRole}
                onToggleRole={onToggleRole}
                primary={runtimeSelection.primaryRole === role.runtimeRole}
                role={role}
                selected={runtimeSelection.selectedRoles.includes(role.runtimeRole)}
              />
            ))}
          </div>
        </>
      ) : null}
    </div>
  )
}

function RoleToggle({
  disabled,
  onToggleRole,
  primary,
  role,
  selected,
}: {
  disabled: boolean
  onToggleRole: (value: string) => void
  primary: boolean
  role: AgentRuntimeRoleOption
  selected: boolean
}) {
  const { t } = useTranslation()

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={() => onToggleRole(role.runtimeRole)}
          className={`flex max-w-40 min-w-0 items-center gap-1.5 rounded-md border px-2.5 py-1.5 text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
        selected
          ? "border-primary/50 bg-primary/10 text-primary"
          : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground"
      }`}
      title={role.capabilities.join(" / ")}
    >
      {selected ? <Check className="size-3.5 shrink-0" /> : null}
      <span className="truncate">{role.name}</span>
      {primary ? (
        <span className="shrink-0 rounded-full bg-current/10 px-1.5 py-0.5 text-[10px]">
          {t("agentChat.composer.runtime.primary")}
        </span>
      ) : null}
    </button>
  )
}
