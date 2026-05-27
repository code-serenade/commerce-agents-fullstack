import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentDispatchFact } from "./agent-process-kind-view-model"
import {
  getDispatchPlanSkills,
  getLatestDispatchPlan,
} from "./agent-process-runtime-view-model"

const SUCCESS_STATUSES = new Set(["success", "ok", "succeeded", "completed"])
const FAILED_STATUSES = new Set(["failed", "not_found", "validation_error"])

export function appendSkillAuditFact(
  facts: AgentDispatchFact[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  const plannedSkills = getPlannedBusinessSkills(runtimeEvents)
  const requestedEvents = runtimeEvents.filter(isBusinessSkillRequestedEvent)
  const resultEvents = runtimeEvents.filter(isBusinessSkillResultEvent)
  const requestedSkills = uniqueStrings(requestedEvents.map((event) => event.skill_id))
  const returnedSkills = uniqueStrings(resultEvents.map((event) => event.skill_id))
  const succeededSkills = uniqueStrings(
    resultEvents.filter(isSuccessfulSkillResult).map((event) => event.skill_id)
  )
  const failedSkills = uniqueStrings(
    resultEvents.filter(isFailedSkillResult).map(formatSkillStatus)
  )
  const pendingSkills = getPendingSkillStatuses(plannedSkills, requestedSkills, resultEvents)

  if (
    plannedSkills.length === 0 &&
    requestedSkills.length === 0 &&
    returnedSkills.length === 0
  ) {
    return
  }

  facts.push({
    detail: t("agentChat.thread.dispatchFacts.skillAudit.detail", {
      failed: formatSkillList(failedSkills),
      pending: formatSkillList(pendingSkills),
      planned: formatSkillList(plannedSkills),
      requested: formatSkillList(requestedSkills),
      succeeded: formatSkillList(succeededSkills),
    }),
    label: t("agentChat.thread.dispatchFacts.skillAudit.label"),
    tone: failedSkills.length > 0 || pendingSkills.length > 0 ? "warning" : "success",
  })
}

function getPlannedBusinessSkills(runtimeEvents: AgentRuntimeEvent[]) {
  const plan = getLatestDispatchPlan(runtimeEvents)
  if (!plan) return []
  return uniqueStrings(getDispatchPlanSkills(plan).filter(isBusinessSkill))
}

function getPendingSkillStatuses(
  plannedSkills: string[],
  requestedSkills: string[],
  resultEvents: SkillResultEvent[]
) {
  const returnedSkills = uniqueStrings(resultEvents.map((event) => event.skill_id))
  const pendingFromResults = resultEvents
    .filter((event) => !isSuccessfulSkillResult(event) && !isFailedSkillResult(event))
    .map(formatSkillStatus)
  const plannedButNotRequested = plannedSkills
    .filter((skill) => !requestedSkills.includes(skill) && !returnedSkills.includes(skill))
    .map((skill) => `${skill}(not_requested)`)
  const requestedButNotReturned = requestedSkills
    .filter((skill) => !returnedSkills.includes(skill))
    .map((skill) => `${skill}(no_result)`)

  return uniqueStrings([
    ...pendingFromResults,
    ...plannedButNotRequested,
    ...requestedButNotReturned,
  ])
}

function formatSkillStatus(event: SkillResultEvent) {
  return `${event.skill_id}(${event.status || "unknown"})`
}

function formatSkillList(skills: string[]) {
  return skills.length > 0 ? skills.join(", ") : "-"
}

function isBusinessSkill(skillId: string) {
  return !skillId.trim().toLowerCase().startsWith("search_")
}

function isSuccessfulSkillResult(event: SkillResultEvent) {
  return SUCCESS_STATUSES.has(event.status.trim().toLowerCase())
}

function isFailedSkillResult(event: SkillResultEvent) {
  return FAILED_STATUSES.has(event.status.trim().toLowerCase())
}

function isBusinessSkillRequestedEvent(event: AgentRuntimeEvent): event is SkillRequestedEvent {
  return event.type === "skill_requested" && isBusinessSkill(event.skill_id)
}

function isBusinessSkillResultEvent(event: AgentRuntimeEvent): event is SkillResultEvent {
  return event.type === "skill_result" && isBusinessSkill(event.skill_id)
}

function uniqueStrings(values: string[]) {
  return Array.from(new Set(values.filter((value) => value.trim().length > 0)))
}

type SkillRequestedEvent = Extract<AgentRuntimeEvent, { type: "skill_requested" }>
type SkillResultEvent = Extract<AgentRuntimeEvent, { type: "skill_result" }>
