import { describe, expect, it } from "vitest"
import {
  mergeSearchEvidencePacketSummaries,
  summarizeSearchEvidencePacket,
} from "./agent-search-evidence-packet-view-model"

describe("search evidence packet view model", () => {
  it("summarizes live search evidence packet quality", () => {
    expect(
      summarizeSearchEvidencePacket({
        evidence_packet: {
          confidence: 0.72,
          gaps: [],
          search_strength: "standard",
          sources: [{ ref: "source 1" }, { ref: "source 2" }],
          usable_claims: ["claim 1"],
        },
      })
    ).toEqual({
      claims: 1,
      confidence: 0.72,
      gaps: 0,
      sources: 2,
      strength: "standard",
    })
  })

  it("summarizes degraded search evidence packet gaps", () => {
    expect(
      summarizeSearchEvidencePacket({
        evidence_packet: {
          confidence: 0,
          gaps: ["provider timeout", "filtered 2 stale items"],
          search_strength: "light",
          sources: [],
          usable_claims: [],
        },
      })
    ).toEqual({
      claims: 0,
      confidence: 0,
      gaps: 2,
      sources: 0,
      strength: "light",
    })
  })

  it("merges multiple packet summaries for one turn", () => {
    expect(
      mergeSearchEvidencePacketSummaries([
        {
          claims: 1,
          confidence: 0.42,
          gaps: 1,
          sources: 2,
          strength: "light",
        },
        {
          claims: 2,
          confidence: 0.76,
          gaps: 0,
          sources: 3,
          strength: "standard",
        },
      ])
    ).toEqual({
      claims: 3,
      confidence: 0.76,
      gaps: 1,
      sources: 5,
      strength: "standard",
    })
  })
})
