import type { AgentRuntimeEvent } from "@/api"

export function resolveWorkerNodeId(
  event: Extract<
    AgentRuntimeEvent,
    { type: "skill_requested" | "skill_result" | "worker_started" | "worker_finished" }
  >
) {
  if ("role" in event && typeof event.role === "string") {
    return runtimeRoleToNodeId(event.role)
  }
  const rawWorkerId = event.workerId ?? event.worker_id ?? ""
  return runtimeRoleToNodeId(rawWorkerId)
}

export function readRuntimeNodeId(
  event: Extract<
    AgentRuntimeEvent,
    {
      type:
        | "task_node_failed"
        | "task_node_passed"
        | "task_node_ready"
        | "task_node_reported"
        | "task_node_rework_requested"
        | "task_node_started"
    }
  >
) {
  return event.nodeId ?? event.node_id ?? ""
}

function runtimeRoleToNodeId(value: string) {
  const rawWorkerId = value.toLowerCase()
  if (rawWorkerId.includes("chat")) return "ops"
  if (rawWorkerId.includes("delivery")) return "delivery"
  if (rawWorkerId.includes("data")) return "data"
  if (rawWorkerId.includes("service")) return "service"
  if (rawWorkerId.includes("engineering")) return "engineering"
  if (rawWorkerId.includes("design")) return "design"
  if (rawWorkerId.includes("search")) return "search"
  if (rawWorkerId.includes("web") || rawWorkerId.includes("seo")) return "web"
  if (rawWorkerId.includes("account")) return "accounting"
  if (
    rawWorkerId.includes("ads") ||
    rawWorkerId.includes("douyin") ||
    rawWorkerId.includes("qianchuan")
  ) {
    return "ops"
  }
  if (rawWorkerId.includes("creative")) return "creative"
  return "ops"
}
