export interface SearchEvidencePacketSummary {
  claims: number
  confidence: number | null
  gaps: number
  sources: number
  strength: string | null
}

export function summarizeSearchEvidencePacket(output: unknown): SearchEvidencePacketSummary | null {
  const safeSummary = summarizeSafeSearchEvidencePacket(output)
  if (safeSummary) return safeSummary

  const packet = readRecord(output)?.evidence_packet
  const record = readRecord(packet)
  if (!record) return null

  return {
    claims: readArray(record.usable_claims).length,
    confidence: readNumber(record.confidence),
    gaps: readArray(record.gaps).length,
    sources: readArray(record.sources).length,
    strength: readString(record.search_strength),
  }
}

export function summarizeSkillOutputEvidence(value: {
  output?: unknown
  outputSummary?: unknown
  output_summary?: unknown
}): SearchEvidencePacketSummary | null {
  return summarizeSearchEvidencePacket(value.outputSummary ?? value.output_summary) ??
    summarizeSearchEvidencePacket(value.output)
}

export function mergeSearchEvidencePacketSummaries(
  summaries: SearchEvidencePacketSummary[]
): SearchEvidencePacketSummary | null {
  if (summaries.length === 0) return null
  const confidences = summaries
    .map((summary) => summary.confidence)
    .filter((value): value is number => typeof value === "number")
  const confidence = confidences.length
    ? Math.max(...confidences)
    : null
  const strength = strongestSearchStrength(summaries.map((summary) => summary.strength))

  return {
    claims: summaries.reduce((total, summary) => total + summary.claims, 0),
    confidence,
    gaps: summaries.reduce((total, summary) => total + summary.gaps, 0),
    sources: summaries.reduce((total, summary) => total + summary.sources, 0),
    strength,
  }
}

function readRecord(value: unknown): Record<string, unknown> | null {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null
}

function readArray(value: unknown) {
  return Array.isArray(value) ? value : []
}

function readNumber(value: unknown) {
  return typeof value === "number" && Number.isFinite(value) ? value : null
}

function readString(value: unknown) {
  return typeof value === "string" && value.trim() ? value : null
}

function summarizeSafeSearchEvidencePacket(outputSummary: unknown) {
  const packet =
    readRecord(outputSummary)?.evidencePacket ??
    readRecord(outputSummary)?.evidence_packet
  const record = readRecord(packet)
  if (!record || record.available === false) return null

  return {
    claims: readCount(record.usableClaims ?? record.usable_claims),
    confidence: readNumber(record.confidence),
    gaps: readCount(record.gaps),
    sources: readCount(record.sources),
    strength: readString(record.searchStrength ?? record.search_strength),
  }
}

function readCount(value: unknown) {
  return typeof value === "number" && Number.isFinite(value)
    ? Math.max(0, Math.trunc(value))
    : readArray(value).length
}

function strongestSearchStrength(values: Array<string | null>) {
  const ranks = new Map([
    ["light", 1],
    ["standard", 2],
    ["deep", 3],
  ])
  return values.reduce<string | null>((best, value) => {
    if (!value) return best
    if (!best) return value
    return (ranks.get(value) ?? 0) > (ranks.get(best) ?? 0) ? value : best
  }, null)
}
