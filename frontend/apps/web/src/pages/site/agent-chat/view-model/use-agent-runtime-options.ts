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
  setRuntimePrimaryRole,
  toggleRuntimeRole,
} from "../domain/agent-runtime-options"
import { agentChatData } from "../agent-chat-data"

interface AgentRuntimeOptionsState {
  setIsLoadingRuntimeOptions: Dispatch<SetStateAction<boolean>>
  setRuntimeOptions: Dispatch<SetStateAction<AgentRuntimeOptions | null>>
  setRuntimeSelection: Dispatch<SetStateAction<AgentChatRuntimeSelection>>
}

export function useAgentRuntimeOptions(input: { enabled: boolean }) {
  const [runtimeOptions, setRuntimeOptions] =
    useState<AgentRuntimeOptions | null>(null)
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
        setRuntimeSelection((current) => ({
          ...current,
          collaborationMode,
          selectedRoles:
            collaborationMode === "single"
              ? current.primaryRole
                ? [current.primaryRole]
                : current.selectedRoles.slice(0, 1)
              : current.selectedRoles,
        }))
      },
      setLearningLevel: (learningLevel: string) => {
        setRuntimeSelection((current) => ({ ...current, learningLevel }))
      },
      setModelName: (modelName: string) => {
        setRuntimeSelection((current) => ({ ...current, modelName }))
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
    runtimeOptions: input.enabled ? runtimeOptions : null,
    runtimeSelection: input.enabled
      ? runtimeSelection
      : defaultRuntimeSelection(null),
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
      state.setRuntimeOptions(null)
    })
    .finally(() => {
      if (!isCancelled()) state.setIsLoadingRuntimeOptions(false)
    })
}
