import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentDispatchFact } from "./agent-process-kind-view-model"

const MATERIAL_REF_PREFIXES = [
  "conversation_file:",
  "product_asset:",
  "product_material:",
  "workspace_artifact:",
]

export function appendMaterialContextFacts(
  facts: AgentDispatchFact[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  const context = getLatestContextLoadedEvent(runtimeEvents)
  const workspace = getLatestWorkspacePackEvent(runtimeEvents)
  const selectedRefs = context ? getSelectedMaterialRefs(context) : []
  const selected = selectedRefs.length
  const packed = workspace?.items?.length ?? 0
  const readable = workspace ? getReadableItemCount(workspace) : 0
  const packedRefs = workspace ? getPackedMaterialRefs(workspace) : []
  const omitted = workspace ? getOmittedRefs(workspace) : []
  const warnings = workspace?.warnings ?? []
  const readiness = workspace?.readiness ?? (selected > 0 ? "pending" : "empty")
  const missingSelected = workspace
    ? getMissingSelectedRefs(selectedRefs, packedRefs, omitted).length
    : selected
  const unreadablePacked = Math.max(packed - readable, 0)
  const summary = getMaterialAuditSummary(
    {
      missingSelected,
      omitted: omitted.length,
      readiness,
      readable,
      selected,
      unreadablePacked,
      warnings: warnings.length,
      workspaceLoaded: Boolean(workspace),
    },
    t
  )

  if (selected > 0 || workspace || omitted.length > 0 || warnings.length > 0) {
    facts.push({
      detail: t("agentChat.thread.dispatchFacts.materialAudit.detail", {
        missing: missingSelected,
        omitted: omitted.length,
        packed,
        readable,
        readiness,
        selected,
        summary,
        unreadable: unreadablePacked,
        warnings: warnings.length,
      }),
      label: t("agentChat.thread.dispatchFacts.materialAudit.label"),
      tone: getMaterialAuditTone({
        missingSelected,
        omitted: omitted.length,
        packed,
        readable,
        readiness,
        selected,
        unreadablePacked,
        warnings: warnings.length,
        workspaceLoaded: Boolean(workspace),
      }),
    })
  }

  if (!workspace) return

  facts.push({
    detail: t("agentChat.thread.dispatchFacts.workspace.detail", {
      readiness,
      readable,
      total: packed,
    }),
    label: t("agentChat.thread.dispatchFacts.workspace.label"),
    tone: readable > 0 ? "success" : "warning",
  })

  const warning = warnings[0]
  if (warning) {
    facts.push({
      detail: t("agentChat.thread.dispatchFacts.workspaceWarning.detail", {
        warning,
      }),
      label: t("agentChat.thread.dispatchFacts.workspaceWarning.label"),
      tone: "warning",
    })
  }

  if (omitted.length > 0) {
    facts.push({
      detail: t("agentChat.thread.dispatchFacts.workspaceOmitted.detail", {
        count: omitted.length,
        refs: omitted.slice(0, 3).join(", "),
      }),
      label: t("agentChat.thread.dispatchFacts.workspaceOmitted.label"),
      tone: "warning",
    })
  }
}

function getLatestContextLoadedEvent(runtimeEvents: AgentRuntimeEvent[]) {
  return [...runtimeEvents]
    .reverse()
    .find((event): event is Extract<AgentRuntimeEvent, { type: "context_loaded" }> =>
      event.type === "context_loaded"
    )
}

function getLatestWorkspacePackEvent(runtimeEvents: AgentRuntimeEvent[]) {
  return [...runtimeEvents]
    .reverse()
    .find(
      (
        event
      ): event is Extract<AgentRuntimeEvent, { type: "workspace_context_pack_loaded" }> =>
        event.type === "workspace_context_pack_loaded"
    )
}

function getSelectedMaterialRefs(
  event: Extract<AgentRuntimeEvent, { type: "context_loaded" }>
) {
  return uniqueStrings([...(event.contextRefs ?? []), ...(event.refs ?? [])]).filter(
    isMaterialRef
  )
}

function getReadableItemCount(
  event: Extract<AgentRuntimeEvent, { type: "workspace_context_pack_loaded" }>
) {
  return (event.items ?? []).filter((item) => Boolean(item.text)).length
}

function getPackedMaterialRefs(
  event: Extract<AgentRuntimeEvent, { type: "workspace_context_pack_loaded" }>
) {
  return uniqueStrings(
    (event.items ?? []).map((item) => item.refKey ?? item.ref_key ?? item.id)
  )
}

function getOmittedRefs(
  event: Extract<AgentRuntimeEvent, { type: "workspace_context_pack_loaded" }>
) {
  const omittedDetails = event.omittedDetails ?? event.omitted_details ?? []
  if (omittedDetails.length > 0) {
    return omittedDetails.map((item) => item.refKey ?? item.ref_key ?? "-")
  }
  return event.omitted ?? []
}

function getMissingSelectedRefs(
  selectedRefs: string[],
  packedRefs: string[],
  omittedRefs: string[]
) {
  return selectedRefs.filter(
    (ref) =>
      !hasComparableMaterialRef(packedRefs, ref) &&
      !hasComparableMaterialRef(omittedRefs, ref)
  )
}

function getMaterialAuditSummary(
  input: {
    missingSelected: number
    omitted: number
    readable: number
    readiness: string
    selected: number
    unreadablePacked: number
    warnings: number
    workspaceLoaded: boolean
  },
  t: TFunction
) {
  if (input.selected > 0 && !input.workspaceLoaded) {
    return t("agentChat.thread.dispatchFacts.materialAudit.summary.pendingPack")
  }
  if (input.missingSelected > 0) {
    return t("agentChat.thread.dispatchFacts.materialAudit.summary.missingPack", {
      missing: input.missingSelected,
    })
  }
  if (input.omitted > 0) {
    return t("agentChat.thread.dispatchFacts.materialAudit.summary.omitted", {
      omitted: input.omitted,
    })
  }
  if (input.unreadablePacked > 0) {
    return t("agentChat.thread.dispatchFacts.materialAudit.summary.unreadablePacked", {
      unreadable: input.unreadablePacked,
    })
  }
  if (input.warnings > 0) {
    return t("agentChat.thread.dispatchFacts.materialAudit.summary.warnings", {
      warnings: input.warnings,
    })
  }
  if (["empty", "partial", "pending"].includes(input.readiness)) {
    return t("agentChat.thread.dispatchFacts.materialAudit.summary.notReady", {
      readiness: input.readiness,
    })
  }
  if (input.readable > 0) {
    return t("agentChat.thread.dispatchFacts.materialAudit.summary.ready")
  }
  return t("agentChat.thread.dispatchFacts.materialAudit.summary.empty")
}

function getMaterialAuditTone(input: {
  missingSelected: number
  omitted: number
  packed: number
  readable: number
  readiness: string
  selected: number
  unreadablePacked: number
  warnings: number
  workspaceLoaded: boolean
}): AgentDispatchFact["tone"] {
  if (
    input.missingSelected > 0 ||
    input.omitted > 0 ||
    input.unreadablePacked > 0 ||
    input.warnings > 0 ||
    (input.selected > 0 && !input.workspaceLoaded) ||
    (input.packed > 0 && input.readable === 0) ||
    ["empty", "partial", "pending"].includes(input.readiness)
  ) {
    return "warning"
  }
  return input.readable > 0 ? "success" : "default"
}

function isMaterialRef(ref: string) {
  const normalized = ref.trim().toLowerCase()
  return MATERIAL_REF_PREFIXES.some((prefix) => normalized.startsWith(prefix))
}

function hasComparableMaterialRef(refs: string[], target: string) {
  return refs.some((ref) => isComparableMaterialRef(ref, target))
}

function isComparableMaterialRef(left: string, right: string) {
  const normalizedLeft = normalizeMaterialRef(left)
  const normalizedRight = normalizeMaterialRef(right)

  return (
    normalizedLeft === normalizedRight ||
    normalizedLeft.endsWith(`:${normalizedRight}`) ||
    normalizedRight.endsWith(`:${normalizedLeft}`)
  )
}

function normalizeMaterialRef(ref: string) {
  return ref.trim().toLowerCase()
}

function uniqueStrings(values: string[]) {
  return Array.from(new Set(values.map((value) => value.trim()).filter(Boolean)))
}
