import type { AgentRuntimeEvent } from "@/api"

export function resolveWorkerNodeId(
  event: Extract<AgentRuntimeEvent, { type: "worker_started" | "worker_finished" }>
) {
  const rawWorkerId = event.workerId ?? event.worker_id ?? ""
  if (rawWorkerId.includes("chat")) return "ops"
  if (rawWorkerId.includes("delivery")) return "delivery"
  if (rawWorkerId.includes("data")) return "data"
  if (rawWorkerId.includes("account")) return "accounting"
  if (rawWorkerId.includes("ads") || rawWorkerId.includes("douyin")) {
    return "douyin_ads"
  }
  if (rawWorkerId.includes("creative")) return "creative"
  return "ops"
}
