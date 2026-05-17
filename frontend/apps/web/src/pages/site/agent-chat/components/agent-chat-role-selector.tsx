import { Crown, UserRound, UsersRound } from "lucide-react"
import { useTranslation } from "react-i18next"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeRoleOption,
} from "../domain/agent-runtime-options"
import {
  PrimaryRoleButton,
  RoleSectionHeader,
  RoleToggle,
  SelectedRoleSummary,
} from "./agent-chat-role-selector-parts"

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
  const isAuto = runtimeSelection.collaborationMode === "auto"
  const isSingle = runtimeSelection.collaborationMode === "single"
  const primaryRole = runtimeSelection.primaryRole ?? roles[0]?.runtimeRole ?? ""
  const supportRoleCount = runtimeSelection.selectedRoles.filter(
    (role) => role !== primaryRole
  ).length
  const selectedSupportRoles = roles.filter(
    (role) =>
      role.runtimeRole !== primaryRole &&
      runtimeSelection.selectedRoles.includes(role.runtimeRole)
  )

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

  if (isSingle) {
    return (
      <div className="grid gap-2 rounded-lg border border-border bg-muted/20 p-2.5">
        <RoleSectionHeader
          icon={<UserRound className="size-3.5 shrink-0" />}
          title={t("agentChat.composer.runtime.singleRole")}
          description={t("agentChat.composer.runtime.singleRoleHint")}
        />
        <div className="grid max-h-56 gap-2 overflow-y-auto pr-1 sm:grid-cols-2 xl:grid-cols-3">
          {roles.map((role) => (
            <PrimaryRoleButton
              badge={t("agentChat.composer.runtime.answerRoleBadge")}
              disabled={disabled}
              key={role.runtimeRole}
              onSelect={onPrimaryRoleChange}
              role={role}
              selected={primaryRole === role.runtimeRole}
            />
          ))}
        </div>
      </div>
    )
  }

  return (
    <div className="grid gap-3">
      <div className="grid gap-2 rounded-lg border border-primary/25 bg-primary/5 p-2.5">
        <RoleSectionHeader
          icon={<Crown className="size-3.5 shrink-0" />}
          title={t("agentChat.composer.runtime.leadRole")}
          description={t("agentChat.composer.runtime.leadRoleHint")}
        />
        <div className="grid max-h-48 gap-2 overflow-y-auto pr-1 sm:grid-cols-2 xl:grid-cols-3">
          {roles.map((role) => (
            <PrimaryRoleButton
              badge={t("agentChat.composer.runtime.primaryBadge")}
              disabled={disabled}
              key={role.runtimeRole}
              onSelect={onPrimaryRoleChange}
              role={role}
              selected={primaryRole === role.runtimeRole}
            />
          ))}
        </div>
      </div>
      <div className="grid gap-2 rounded-lg border border-border bg-card p-2.5">
        <RoleSectionHeader
          icon={<UsersRound className="size-3.5 shrink-0" />}
          title={t("agentChat.composer.runtime.supportRoles")}
          description={t("agentChat.composer.runtime.supportRolesHint")}
        />
        <SelectedRoleSummary
          primaryRole={roles.find((role) => role.runtimeRole === primaryRole)}
          supportRoles={selectedSupportRoles}
        />
        <div className="flex max-h-40 min-w-0 flex-wrap gap-1.5 overflow-y-auto pr-1">
          {roles
            .filter((role) => role.runtimeRole !== primaryRole)
            .map((role) => {
              const selected = runtimeSelection.selectedRoles.includes(
                role.runtimeRole
              )
              return (
                <RoleToggle
                  disabled={disabled || (selected && supportRoleCount <= 1)}
                  key={role.runtimeRole}
                  onToggleRole={onToggleRole}
                  role={role}
                  selected={selected}
                />
              )
            })}
        </div>
      </div>
    </div>
  )
}
