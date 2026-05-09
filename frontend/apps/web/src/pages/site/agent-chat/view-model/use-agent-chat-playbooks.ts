import { useEffect, useMemo, useState } from "react"
import type { AgentChatPlaybookSelection } from "../domain/agent-chat-playbook"
import {
  loadAgentChatPlaybooks,
  resetAgentChatPlaybooks,
} from "./agent-chat-playbook-actions"

export function useAgentChatPlaybooks(input: { enabled: boolean }) {
  const [playbooks, setPlaybooks] = useState<AgentChatPlaybookSelection[]>([])
  const [isLoadingPlaybooks, setIsLoadingPlaybooks] = useState(false)
  const state = useMemo(
    () => ({
      setIsLoadingPlaybooks,
      setPlaybooks,
    }),
    []
  )

  useEffect(() => {
    let cancelled = false

    if (!input.enabled) {
      resetAgentChatPlaybooks(state)
      return
    }

    void loadAgentChatPlaybooks(state, () => cancelled)

    return () => {
      cancelled = true
    }
  }, [input.enabled, state])

  return {
    isLoadingPlaybooks,
    playbooks,
  }
}
