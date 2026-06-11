import type { AgentRuntimeEvent } from "@/api"

type WorkspaceContextPackEvent = Extract<
  AgentRuntimeEvent,
  { type: "workspace_context_pack_loaded" }
>

export function getWorkspaceContextPackItemCount(
  event: WorkspaceContextPackEvent
) {
  return Math.max(event.itemCount ?? event.item_count ?? event.items?.length ?? 0, 0)
}

export function getWorkspaceContextPackReadableCount(
  event: WorkspaceContextPackEvent
) {
  return Math.max(
    event.readableItemCount ??
      event.readable_item_count ??
      (event.items ?? []).filter(
        (item) => Boolean(item.text) || (item.textChars ?? item.text_chars ?? 0) > 0
      ).length,
    0
  )
}

export function getWorkspaceContextPackUnreadableCount(
  event: WorkspaceContextPackEvent
) {
  return Math.max(
    getWorkspaceContextPackItemCount(event) -
      getWorkspaceContextPackReadableCount(event),
    0
  )
}

export function hasWorkspaceContextPackItemRefs(event: WorkspaceContextPackEvent) {
  return (event.items ?? []).some((item) => item.refKey ?? item.ref_key ?? item.id)
}
