import {
  BadgeDollarSign,
  BarChart3,
  ClipboardList,
  Loader2,
  PenLine,
  Rocket,
  ShieldAlert,
  type LucideIcon,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentChatPlaybookSelection } from "../domain/agent-chat-playbook"

const PLAYBOOK_ICONS: Record<string, LucideIcon> = {
  briefing: ClipboardList,
  creative: PenLine,
  launch: Rocket,
  pricing: BadgeDollarSign,
  review: BarChart3,
  service: ShieldAlert,
}

export interface AgentChatPlaybookButtonsProps {
  isLoading: boolean
  isStreaming: boolean
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  playbooks: AgentChatPlaybookSelection[]
}

export function AgentChatPlaybookButtons({
  isLoading,
  isStreaming,
  onSelectPlaybook,
  playbooks,
}: AgentChatPlaybookButtonsProps) {
  const { t } = useTranslation()

  return (
    <div className="flex flex-wrap gap-2">
      {isLoading ? (
        <span className="flex h-7 items-center gap-2 rounded-full border border-border bg-background/70 px-3 text-xs text-muted-foreground">
          <Loader2 className="size-3.5 animate-spin" />
          {t("agentChat.composer.playbooks.loading")}
        </span>
      ) : null}
      {!isLoading && playbooks.length === 0 ? (
        <span className="flex h-7 items-center rounded-full border border-border bg-background/70 px-3 text-xs text-muted-foreground">
          {t("agentChat.composer.playbooks.empty")}
        </span>
      ) : null}
      {playbooks.map((playbook) => {
        const Icon = PLAYBOOK_ICONS[playbook.category] ?? ClipboardList
        return (
          <button
            key={playbook.id}
            type="button"
            disabled={isStreaming}
            onClick={() => onSelectPlaybook(playbook)}
            className="flex max-w-44 items-center gap-2 rounded-full border border-border bg-background px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
            title={`${playbook.description} ${playbook.expectedOutputs.join(" / ")}`}
          >
            <Icon className="size-3.5" />
            <span className="truncate">{playbook.title}</span>
          </button>
        )
      })}
    </div>
  )
}
