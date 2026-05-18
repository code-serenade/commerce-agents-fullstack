import type { Dispatch, SetStateAction } from "react"
import { agentChatData } from "../agent-chat-data"
import {
  mapCommercePlaybookToSelection,
  type AgentChatPlaybookSelection,
} from "../domain/agent-chat-playbook"

export interface AgentChatPlaybookState {
  setIsLoadingPlaybooks: Dispatch<SetStateAction<boolean>>
  setPlaybooks: Dispatch<SetStateAction<AgentChatPlaybookSelection[]>>
}

export function resetAgentChatPlaybooks(state: AgentChatPlaybookState) {
  state.setPlaybooks([])
  state.setIsLoadingPlaybooks(false)
}

export function loadAgentChatPlaybooks(
  state: AgentChatPlaybookState,
  isCancelled: () => boolean
) {
  state.setIsLoadingPlaybooks(true)
  return agentChatData
    .listPlaybooks()
    .then((playbooks) => {
      if (isCancelled()) return
      state.setPlaybooks(playbooks.map(mapCommercePlaybookToSelection))
    })
    .catch(() => {
      if (isCancelled()) return
      state.setPlaybooks([])
    })
    .finally(() => {
      if (!isCancelled()) state.setIsLoadingPlaybooks(false)
    })
}
