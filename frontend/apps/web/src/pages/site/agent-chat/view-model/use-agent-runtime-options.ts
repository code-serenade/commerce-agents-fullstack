import type { Dispatch, SetStateAction } from "react"
import { useEffect, useMemo, useState } from "react"
import type {
  AgentChatRuntimeSelectionRestore,
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import {
  defaultRuntimeSelection,
  normalizeRuntimeSelection,
  restoreRuntimeSelection,
  setRuntimeCollaborationMode,
  setRuntimePrimaryRole,
  setRuntimeRoutingMode,
  toggleRuntimeRole,
} from "../domain/agent-runtime-options"
import { DEFAULT_AGENT_RUNTIME_OPTIONS } from "../domain/default-agent-runtime-options"
import { agentChatData } from "../agent-chat-data"

interface AgentRuntimeOptionsState {
  setIsLoadingRuntimeOptions: Dispatch<SetStateAction<boolean>>
  setRuntimeOptions: Dispatch<SetStateAction<AgentRuntimeOptions | null>>
  setRuntimeSelection: Dispatch<SetStateAction<AgentChatRuntimeSelection>>
}

export function useAgentRuntimeOptions(input: { enabled: boolean }) {
  const [runtimeOptions, setRuntimeOptions] =
    useState<AgentRuntimeOptions | null>(DEFAULT_AGENT_RUNTIME_OPTIONS)
  const [isLoadingRuntimeOptions, setIsLoadingRuntimeOptions] = useState(false)
  const [runtimeSelection, setRuntimeSelection] =
    useState<AgentChatRuntimeSelection>(() => defaultRuntimeSelection(null))
  const state = useMemo(
    () => ({
      setIsLoadingRuntimeOptions,
      setRuntimeOptions,
      setRuntimeSelection,
    }),
    []
  )

  useEffect(() => {
    let cancelled = false

    if (!input.enabled) {
      return
    }

    void loadAgentRuntimeOptions(state, () => cancelled)

    return () => {
      cancelled = true
    }
  }, [input.enabled, state])

  const commands = useMemo(
    () => ({
      setCollaborationMode: (collaborationMode: string) => {
        setRuntimeSelection((current) =>
          setRuntimeCollaborationMode(current, collaborationMode)
        )
      },
      setLearningLevel: (learningLevel: string) => {
        setRuntimeSelection((current) => ({ ...current, learningLevel }))
      },
      setRoutingMode: (routingMode: string) => {
        setRuntimeSelection((current) =>
          setRuntimeRoutingMode(current, routingMode)
        )
      },
      setPrimaryRole: (primaryRole: string) => {
        setRuntimeSelection((current) =>
          setRuntimePrimaryRole(current, primaryRole)
        )
      },
      restoreSelection: (selection: AgentChatRuntimeSelectionRestore) => {
        setRuntimeSelection((current) =>
          restoreRuntimeSelection(current, selection, runtimeOptions)
        )
      },
      resetSelection: () => {
        setRuntimeSelection(defaultRuntimeSelection(runtimeOptions))
      },
      toggleRole: (runtimeRole: string) => {
        setRuntimeSelection((current) => toggleRuntimeRole(current, runtimeRole))
      },
    }),
    [runtimeOptions]
  )

  return {
    isLoadingRuntimeOptions: input.enabled && isLoadingRuntimeOptions,
    runtimeOptions: runtimeOptions ?? DEFAULT_AGENT_RUNTIME_OPTIONS,
    runtimeSelection,
    ...commands,
  }
}

function loadAgentRuntimeOptions(
  state: AgentRuntimeOptionsState,
  isCancelled: () => boolean
) {
  state.setIsLoadingRuntimeOptions(true)
  return agentChatData
    .getRuntimeOptions()
    .then((options) => {
      if (isCancelled()) return
      state.setRuntimeOptions(options)
      state.setRuntimeSelection((current) =>
        normalizeRuntimeSelection(current, options)
      )
    })
    .catch(() => {
      if (isCancelled()) return
      state.setRuntimeOptions(DEFAULT_AGENT_RUNTIME_OPTIONS)
      state.setRuntimeSelection((current) =>
        normalizeRuntimeSelection(current, DEFAULT_AGENT_RUNTIME_OPTIONS)
      )
    })
    .finally(() => {
      if (!isCancelled()) state.setIsLoadingRuntimeOptions(false)
    })
}
