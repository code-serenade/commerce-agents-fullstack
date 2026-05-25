import { useTranslation } from "react-i18next"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeRoleOption,
} from "../domain/agent-runtime-options"
import {
  RoleAssignmentRow,
  RoleChoiceButton,
} from "./agent-chat-role-selector-parts"
import { SettingsRow } from "./agent-chat-runtime-shared"

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
  const isManual = runtimeSelection.collaborationMode === "manual"
  const primaryRole = runtimeSelection.primaryRole ?? roles[0]?.runtimeRole ?? ""

  if (isAuto) {
    return null
  }

  if (isManual) {
    return (
      <SettingsRow
        align="start"
        label={t("agentChat.composer.runtime.selectedRoles")}
      >
        <div className="grid min-w-0 gap-1.5">
          <div className="grid grid-cols-[minmax(0,1fr)_6rem_5rem] items-center gap-1.5 px-2 text-xs font-medium text-muted-foreground">
            <span className="truncate">
              {t("agentChat.composer.runtime.roleColumn")}
            </span>
            <span className="text-center leading-tight">
              {t("agentChat.composer.runtime.primaryShort")}
            </span>
            <span className="text-center leading-tight">
              {t("agentChat.composer.runtime.supportShort")}
            </span>
          </div>
          {roles.map((role) => {
            const isPrimary = primaryRole === role.runtimeRole
            return (
              <RoleAssignmentRow
                disabled={disabled}
                isPrimary={isPrimary}
                isSupport={
                  !isPrimary &&
                  runtimeSelection.selectedRoles.includes(role.runtimeRole)
                }
                key={role.runtimeRole}
                onPrimaryRoleChange={onPrimaryRoleChange}
                onToggleRole={onToggleRole}
                role={role}
              />
            )
          })}
        </div>
      </SettingsRow>
    )
  }

  return (
    <div className="grid gap-2">
      <SettingsRow
        align="start"
        label={t("agentChat.composer.runtime.singleRole")}
      >
        <div className="grid min-w-0 gap-1.5 sm:grid-cols-2 xl:grid-cols-3">
          {roles.map((role) => {
            const primary = primaryRole === role.runtimeRole
            return (
              <RoleChoiceButton
                disabled={disabled}
                key={role.runtimeRole}
                onClick={onPrimaryRoleChange}
                role={role}
                selected={primary}
                variant="lead"
              />
            )
          })}
        </div>
      </SettingsRow>
    </div>
  )
}
