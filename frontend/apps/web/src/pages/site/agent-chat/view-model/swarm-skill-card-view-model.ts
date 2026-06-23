import { Wrench } from "lucide-react"
import type { TFunction } from "i18next"
import type { AgentSkillRuntimeResult } from "./swarm-runtime-state-view-model"
import type { ComputerCard } from "./swarm-panel-view-model"
import type { RuntimeNodeSnapshot } from "./swarm-runtime-state-view-model"
import { formatUserFacingSkillStatus } from "./agent-user-facing-runtime-text"
import { formatTimelineDynamicText } from "./swarm-timeline-user-facing-text"

export function getSkillCard(
  activeNode: RuntimeNodeSnapshot | undefined,
  t: TFunction
): ComputerCard | null {
  const latestSkill = activeNode?.skillResults.at(-1)
  if (!latestSkill) return null

  return {
    id: `skill-${latestSkill.skillId}`,
    title: t("agentChat.swarm.cards.skill.title"),
    meta: t("agentChat.swarm.cards.skill.meta"),
    description: formatSkillDescription(latestSkill, t),
    icon: Wrench,
    lines: [
      t("agentChat.swarm.cards.skill.statusLine", {
        status: formatUserFacingSkillStatus(latestSkill.status, t),
      }),
      formatTimelineDynamicText(
        latestSkill.summary ?? latestSkill.error ?? latestSkill.reason,
        t
      ),
    ].filter(Boolean),
    state: skillState(latestSkill.status, latestSkill.boundary),
  }
}

function formatSkillDescription(
  skill: AgentSkillRuntimeResult,
  t: TFunction
) {
  if (skill.status === "requested") {
    return t("agentChat.swarm.cards.skill.requested")
  }
  if (skill.approvalRequired) {
    return t("agentChat.swarm.cards.skill.approvalRequired")
  }
  if (skill.boundary) {
    return skill.summary
      ? formatTimelineDynamicText(skill.summary, t)
      : t("agentChat.swarm.cards.skill.completed")
  }
  if (skill.status === "deferred") {
    return t("agentChat.swarm.cards.skill.deferred")
  }
  if (skill.error) return formatTimelineDynamicText(skill.error, t)
  return skill.summary
    ? formatTimelineDynamicText(skill.summary, t)
    : t("agentChat.swarm.cards.skill.completed")
}

function skillState(status: string, boundary?: boolean) {
  if (boundary) return "boundary" as const
  if (status === "success") return "done" as const
  if (status === "requested" || status === "deferred" || status === "pending_approval") {
    return "working" as const
  }
  if (status === "failed" || status === "not_found" || status === "validation_error") {
    return "failed" as const
  }
  return "queued" as const
}
