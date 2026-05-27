import { useEffect, useMemo } from "react"
import { useSearchParams } from "react-router"
import {
  clearComposerPanelSearchParam,
  requestedComposerPanelFromSearch,
  type RequestedComposerPanelState,
} from "./agent-chat-composer-panel-url"

const initialRequestedComposerPanelState: RequestedComposerPanelState = {
  focusVersion: 0,
  panel: null,
  version: 0,
}

export function useAgentChatRequestedComposerPanel() {
  const [searchParams, setSearchParams] = useSearchParams()
  const requestState = useMemo<RequestedComposerPanelState>(() => {
    const panel = requestedComposerPanelFromSearch(searchParams)
    if (!panel) return initialRequestedComposerPanelState

    return {
      focusVersion: 1,
      panel,
      version: 1,
    }
  }, [searchParams])

  useEffect(() => {
    if (searchParams.get("panel") !== "materials") return
    setSearchParams(clearComposerPanelSearchParam(searchParams), {
      replace: true,
    })
  }, [searchParams, setSearchParams])

  return requestState
}
