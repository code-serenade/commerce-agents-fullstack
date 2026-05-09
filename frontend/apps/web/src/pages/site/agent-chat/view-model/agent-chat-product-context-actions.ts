import type { Dispatch, SetStateAction } from "react"
import { agentChatData } from "../agent-chat-data"
import {
  mapProductToContextSelection,
  toggleProductContextSelection,
  type AgentChatProductContextSelection,
} from "../domain/agent-chat-context-selection"

export interface AgentChatProductContextState {
  setAvailableProducts: Dispatch<SetStateAction<AgentChatProductContextSelection[]>>
  setIsLoadingProducts: Dispatch<SetStateAction<boolean>>
  setSelectedProducts: Dispatch<SetStateAction<AgentChatProductContextSelection[]>>
}

export function resetAgentChatProductContext(state: AgentChatProductContextState) {
  state.setAvailableProducts([])
  state.setSelectedProducts([])
  state.setIsLoadingProducts(false)
}

export function loadAgentChatProductContext(
  state: AgentChatProductContextState,
  isCancelled: () => boolean
) {
  state.setIsLoadingProducts(true)
  return agentChatData
    .listProducts({ status: "active" })
    .then((products) => {
      if (isCancelled()) return
      const mapped = products.map(mapProductToContextSelection)
      state.setAvailableProducts(mapped)
      state.setSelectedProducts((current) =>
        current.filter((selected) =>
          mapped.some((product) => product.id === selected.id)
        )
      )
    })
    .catch(() => {
      if (isCancelled()) return
      state.setAvailableProducts([])
      state.setSelectedProducts([])
    })
    .finally(() => {
      if (!isCancelled()) state.setIsLoadingProducts(false)
    })
}

export function toggleAgentChatProductContext(
  state: Pick<AgentChatProductContextState, "setSelectedProducts">,
  product: AgentChatProductContextSelection
) {
  state.setSelectedProducts((current) =>
    toggleProductContextSelection(current, product)
  )
}
