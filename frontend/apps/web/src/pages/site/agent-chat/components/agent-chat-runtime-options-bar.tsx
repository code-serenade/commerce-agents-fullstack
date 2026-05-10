import { Check, Cpu, GraduationCap, UsersRound } from "lucide-react"
import type { ReactNode } from "react"
import { useTranslation } from "react-i18next"
import { NativeSelect } from "@workspace/ui-components/stable/native-select"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeChoice,
  AgentRuntimeModelOption,
  AgentRuntimeOptions,
  AgentRuntimeRoleOption,
} from "../domain/agent-runtime-options"

export interface AgentChatRuntimeOptionsBarProps {
  disabled: boolean
  isLoadingRuntimeOptions: boolean
  onCollaborationModeChange: (value: string) => void
  onLearningLevelChange: (value: string) => void
  onModelChange: (value: string) => void
  onPrimaryRoleChange: (value: string) => void
  onToggleRole: (value: string) => void
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
}

export function AgentChatRuntimeOptionsBar({
  disabled,
  isLoadingRuntimeOptions,
  onCollaborationModeChange,
  onLearningLevelChange,
  onModelChange,
  onPrimaryRoleChange,
  onToggleRole,
  runtimeOptions,
  runtimeSelection,
}: AgentChatRuntimeOptionsBarProps) {
  const { t } = useTranslation()

  if (!runtimeOptions && !isLoadingRuntimeOptions) return null

  const controlDisabled = disabled || isLoadingRuntimeOptions

  return (
    <div className="flex min-w-0 flex-wrap items-center gap-2 rounded-2xl bg-background p-2">
      <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
        <Cpu className="size-3.5" />
        {t("agentChat.composer.runtime.label")}
      </span>
      {isLoadingRuntimeOptions ? (
        <span className="rounded-full border border-border bg-background/70 px-3 py-1.5 text-xs text-muted-foreground">
          {t("agentChat.composer.runtime.loading")}
        </span>
      ) : null}
      {runtimeOptions ? (
        <>
          <RuntimeSelect
            disabled={controlDisabled}
            label={t("agentChat.composer.runtime.model")}
            onValueChange={onModelChange}
            options={runtimeOptions.models.map(modelToSelectOption)}
            value={runtimeSelection.modelName ?? ""}
          />
          <RuntimeSelect
            disabled={controlDisabled}
            icon={<GraduationCap className="size-3.5" />}
            label={t("agentChat.composer.runtime.learning")}
            onValueChange={onLearningLevelChange}
            options={choiceToSelectOptions(runtimeOptions.learningLevels)}
            value={runtimeSelection.learningLevel}
          />
          <RuntimeSelect
            disabled={controlDisabled}
            icon={<UsersRound className="size-3.5" />}
            label={t("agentChat.composer.runtime.collaboration")}
            onValueChange={onCollaborationModeChange}
            options={choiceToSelectOptions(runtimeOptions.collaborationModes)}
            value={runtimeSelection.collaborationMode}
          />
          <RoleSelector
            disabled={controlDisabled}
            onPrimaryRoleChange={onPrimaryRoleChange}
            onToggleRole={onToggleRole}
            roles={runtimeOptions.roles}
            runtimeSelection={runtimeSelection}
          />
        </>
      ) : null}
    </div>
  )
}

function RuntimeSelect({
  disabled,
  icon,
  label,
  onValueChange,
  options,
  value,
}: {
  disabled: boolean
  icon?: ReactNode
  label: string
  onValueChange: (value: string) => void
  options: { label: string; value: string }[]
  value: string
}) {
  return (
    <label className="flex items-center gap-1.5 text-xs text-muted-foreground">
      {icon}
      <span>{label}</span>
      <NativeSelect
        disabled={disabled || options.length === 0}
        onValueChange={onValueChange}
        options={options}
        value={value}
      />
    </label>
  )
}

function RoleSelector({
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
  const isSingle = runtimeSelection.collaborationMode === "single"
  const roleOptions = roles.map(roleToSelectOption)

  return (
    <div className="flex min-w-0 flex-wrap items-center gap-1.5">
      <span className="text-xs text-muted-foreground">
        {t("agentChat.composer.runtime.roles")}
      </span>
      {!isSingle ? (
        <RuntimeSelect
          disabled={disabled}
          label={t("agentChat.composer.runtime.primary")}
          onValueChange={onPrimaryRoleChange}
          options={roleOptions}
          value={runtimeSelection.primaryRole ?? ""}
        />
      ) : null}
      {roles.map((role) => {
        const selected = runtimeSelection.selectedRoles.includes(role.runtimeRole)
        const primary = runtimeSelection.primaryRole === role.runtimeRole
        const active = isSingle ? primary : selected

        return (
          <button
            key={role.runtimeRole}
            type="button"
            disabled={disabled}
            onClick={() => {
              if (isSingle) {
                onPrimaryRoleChange(role.runtimeRole)
                return
              }
              onToggleRole(role.runtimeRole)
            }}
            className={`flex max-w-36 min-w-0 items-center gap-1.5 rounded-full border px-2.5 py-1.5 text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
              active
                ? "border-blue-300 bg-blue-50 text-blue-700 dark:border-blue-700/70 dark:bg-blue-950/40 dark:text-blue-100"
                : "border-border bg-background/70 text-muted-foreground hover:bg-muted hover:text-foreground"
            }`}
            title={role.capabilities.join(" / ")}
          >
            {active ? <Check className="size-3.5 shrink-0" /> : null}
            <span className="truncate">{role.name}</span>
            {primary ? (
              <span className="shrink-0 rounded-full bg-current/10 px-1.5 py-0.5 text-[10px]">
                {t("agentChat.composer.runtime.primary")}
              </span>
            ) : null}
          </button>
        )
      })}
    </div>
  )
}

function choiceToSelectOptions(choices: AgentRuntimeChoice[]) {
  return choices.map((choice) => ({
    label: choice.label,
    value: choice.id,
  }))
}

function modelToSelectOption(model: AgentRuntimeModelOption) {
  return {
    label: `${model.name} / ${model.model}`,
    value: model.name,
  }
}

function roleToSelectOption(role: AgentRuntimeRoleOption) {
  return {
    label: role.name,
    value: role.runtimeRole,
  }
}
