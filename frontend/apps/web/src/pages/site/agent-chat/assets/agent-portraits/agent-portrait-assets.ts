import accounting from "./realistic/accounting.png"
import box from "./realistic/box.png"
import creative from "./realistic/creative.png"
import data from "./realistic/data.png"
import delivery from "./realistic/delivery.png"
import design from "./realistic/design.png"
import engineering from "./realistic/engineering.png"
import ops from "./realistic/ops.png"
import search from "./realistic/search.png"
import service from "./realistic/service.png"
import web from "./realistic/web.png"
import homeCreative from "./home-showcase/creative.png"
import homeData from "./home-showcase/data.png"
import homeDesign from "./home-showcase/design.png"
import homeOps from "./home-showcase/ops.png"
import homeSearch from "./home-showcase/search.png"
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

const homeShowcaseAgentPortraits = {
  creative: homeCreative,
  data: homeData,
  design: homeDesign,
  ops: homeOps,
  search: homeSearch,
} satisfies Partial<Record<AgentRoleKey, string>>

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

export function getHomeShowcaseAgentPortraitAsset(
  role: AgentRoleKey
): AgentPortraitAsset | null {
  const asset = homeShowcaseAgentPortraits[role]

  if (!asset) return null

  return {
    key: `${role}-home-showcase-portrait`,
    role,
    src: asset,
  }
}
