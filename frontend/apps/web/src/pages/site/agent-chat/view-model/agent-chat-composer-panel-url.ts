import type { AgentChatComposerPanelId } from "./use-agent-chat-panel-state"

export interface RequestedComposerPanelState {
  focusVersion: number
  panel: AgentChatComposerPanelId | null
  version: number
}

export function requestedComposerPanelFromSearch(
  searchParams: URLSearchParams
): AgentChatComposerPanelId | null {
  return searchParams.get("panel") === "materials" ? "source" : null
}

export function clearComposerPanelSearchParam(
  searchParams: URLSearchParams
): URLSearchParams {
  const nextSearchParams = new URLSearchParams(searchParams)
  nextSearchParams.delete("panel")
  return nextSearchParams
}

export function applyRequestedComposerPanelFromSearch(
  state: RequestedComposerPanelState,
  searchParams: URLSearchParams
): RequestedComposerPanelState {
  const panel = requestedComposerPanelFromSearch(searchParams)
  if (!panel) return state

  return {
    focusVersion: state.focusVersion + 1,
    panel,
    version: state.version + 1,
  }
}
