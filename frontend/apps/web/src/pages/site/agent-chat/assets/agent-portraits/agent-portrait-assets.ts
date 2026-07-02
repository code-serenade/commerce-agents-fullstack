import accounting from "./pixel-selected/accounting.png"
import box from "./pixel-selected/box.png"
import creative from "./pixel-selected/creative.png"
import data from "./pixel-selected/data.png"
import delivery from "./pixel-selected/delivery.png"
import design from "./pixel-selected/design.png"
import engineering from "./pixel-selected/engineering.png"
import ops from "./pixel-selected/ops.png"
import search from "./pixel-selected/search.png"
import service from "./pixel-selected/service.png"
import web from "./pixel-selected/web.png"
import type { AgentRoleKey } from "../../view-model/agent-role-cluster-view-model"

export interface AgentPortraitAsset {
  key: string
  role: AgentRoleKey
  src: string
}

const agentPortraits = {
  accounting,
  box,
  creative,
  data,
  delivery,
  design,
  engineering,
  ops,
  search,
  service,
  web,
} satisfies Record<AgentRoleKey, string>

export function getAgentPortraitAsset(
  role: AgentRoleKey
): AgentPortraitAsset | null {
  const asset = agentPortraits[role]

  if (!asset) return null

  return {
    key: `${role}-portrait`,
    role,
    src: asset,
  }
}
