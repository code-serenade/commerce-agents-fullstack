import { describe, expect, it } from "vitest"
import { formatAgentUiMessage } from "./agent-chat-message-mapping"

const copy = {
  boundaryNotice: "boundary notice",
  missingStructuredOutput: "missing structured output",
  streamTimedOut: "stream timed out",
}

describe("agent answer Edge visible sunscreen tail fallback", () => {
  it("removes repeated summary tail after a complete business answer", () => {
    const keyJudgement = t(20851, 38190, 21028, 26029)
    const executionPlan = t(25191, 34892, 26041, 26696)
    const riskBoundary = t(39118, 38505, 36793, 30028)
    const nextOptional = t(
      19979, 19968, 27493, 65288, 21487, 36873, 65292, 33021, 35753,
      25928, 26524, 26356, 20934, 65289
    )
    const directTitle = t(
      21487, 30452, 25509, 29992, 30340, 36890, 29992, 33258, 28982,
      26631, 39064
    )
    const title = t(
      85, 80, 70, 53, 48, 43, 20937, 24863, 38450, 26194, 34915,
      22799, 23395, 38450, 32043, 22806, 32447
    )
    const repeatedMissing = t(
      24403, 21069, 32570, 23569, 26087, 26631, 39064, 12289, 20027,
      22270, 32454, 33410, 12289, 26333, 20809, 28857, 20987, 25968,
      25454, 12289, 20215, 26684, 23450, 20301, 12289, 21806, 21334,
      24179, 21488
    )
    const repeatedNeeds = t(
      38450, 26194, 34915, 29992, 25143, 26680, 24515, 25628, 32034,
      38656, 27714
    )
    const repeatedCovered = t(
      29616, 26377, 36890, 29992, 26631, 39064, 24050, 35206, 30422,
      26680, 24515, 38656, 27714
    )
    const repeatedRisk = t(
      26631, 39064, 26631, 27880, 30340, 38450, 26194, 25351, 25968,
      19982, 21830, 21697, 23454, 38469, 19981, 31526
    )

    const raw = [
      `## ${keyJudgement}`,
      t(
        20320, 30340, 21407, 26377, 26631, 39064, 22823, 27010, 29575,
        27809, 35828, 28165, 29992, 25143, 20851, 24515, 30340, 30828,
        25351, 26631, 21644, 22330, 26223
      ),
      `## ${executionPlan}`,
      `${directTitle}：${title}`,
      `## ${riskBoundary}`,
      t(
        29616, 22312, 27809, 30475, 21040, 20320, 30340, 26087, 26631,
        39064, 12289, 20027, 22270, 12289, 20855, 20307, 23450, 20215
      ),
      `## ${nextOptional}`,
      t(20320, 21487, 20197, 34917, 20805, 21407, 26631, 39064),
      repeatedMissing,
      repeatedNeeds,
      repeatedCovered,
      repeatedRisk,
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).toContain(keyJudgement)
    expect(formatted).toContain(title)
    expect(formatted).toContain(riskBoundary)
    expect(formatted).toContain(nextOptional)
    expect(formatted).not.toContain(repeatedMissing)
    expect(formatted).not.toContain(repeatedNeeds)
    expect(formatted).not.toContain(repeatedCovered)
    expect(formatted).not.toContain(repeatedRisk)
  })
})

function t(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
