import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import { getDispatchPlanAdaptiveProfile } from "./agent-process-runtime-view-model"
import type { AgentDispatchFact } from "./agent-process-kind-view-model"

export function getAdaptiveDispatchFact(
  plan: Extract<AgentRuntimeEvent, { type: "dispatch_planned" }>,
  t: TFunction
): AgentDispatchFact | null {
  const profile = getDispatchPlanAdaptiveProfile(plan)
  if (!profile) return null

  return {
    detail:
      profile.studentSummary ??
      profile.student_summary ??
      t("agentChat.thread.dispatchFacts.adaptive.detail", {
        confidence: profile.confidence ?? "-",
        level: profile.level ?? plan.difficulty,
        pressure: profile.contextPressure ?? profile.context_pressure ?? "-",
        roles: profile.roleCount ?? profile.role_count ?? 0,
        skills: profile.skillCount ?? profile.skill_count ?? 0,
        steps: profile.stepCount ?? profile.step_count ?? 0,
      }),
    label: t("agentChat.thread.dispatchFacts.adaptive.label"),
    tone: "success",
  }
}
