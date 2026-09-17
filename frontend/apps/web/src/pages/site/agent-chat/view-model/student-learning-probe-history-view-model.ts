import type { LearningProbeResponse, LearningProbeResultResponse } from "@/api"
import type { StoredEngagedProbe } from "./student-learning-probe-storage"

export type DisplayableProbe = LearningProbeResponse | StoredEngagedProbe

export interface CompletedLearningProbe {
  probe: DisplayableProbe
  result: LearningProbeResultResponse
}

export function mergeCompletedLearningProbes(
  serverItems: CompletedLearningProbe[],
  localItems: CompletedLearningProbe[],
  visibleProbeId?: string
) {
  const byId = new Map<string, CompletedLearningProbe>()
  for (const item of serverItems) byId.set(item.probe.id, item)
  for (const item of localItems) byId.set(item.probe.id, item)
  if (visibleProbeId) byId.delete(visibleProbeId)
  return Array.from(byId.values())
}
