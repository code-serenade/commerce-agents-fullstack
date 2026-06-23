import { describe, expect, it } from "vitest"
import { formatAgentUiMessage } from "./agent-chat-message-mapping"

const copy = {
  boundaryNotice: "boundary notice",
  missingStructuredOutput: "missing structured output",
  streamTimedOut: "stream timed out",
}

describe("legacy runtime copy sanitizer", () => {
  it("rewrites old persisted collaboration wording into user-facing processing wording", () => {
    const oldUpstreamOutput = t(19978, 28216, 23703, 20301, 36755, 20986)
    const oldCollaborationStep = t(21327, 20316, 27493, 39588)
    const internalFailureDetail = t(20869, 37096, 22833, 36133, 32454, 33410)
    const existingInfo = t(24050, 26377, 20449, 24687)
    const processStep = t(22788, 29702, 27493, 39588)
    const raw = [
      t(
        20808, 25353, 24403, 21069, 38382, 39064, 12289, 24050,
        36873, 19978, 19979, 25991, 21644, 19978, 28216, 23703,
        20301, 36755, 20986, 32473, 20986, 21487, 25191, 34892,
        33609, 26696, 12290
      ),
      t(
        37096, 20998, 21327, 20316, 27493, 39588, 27809, 26377,
        24418, 25104, 21487, 29992, 32467, 26524, 65292, 25152,
        20197, 20855, 20307, 38144, 37327, 19981, 33021, 20889,
        25104, 30830, 23450, 20107, 23454, 12290
      ),
      t(
        26412, 36718, 26377, 22810, 20010, 21327, 20316, 27493,
        39588, 26410, 23436, 25104, 65292, 26368, 32456, 31572,
        26696, 24050, 36991, 20813, 26292, 38706, 20869, 37096,
        22833, 36133, 32454, 33410, 12290
      ),
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).toContain(existingInfo)
    expect(formatted).toContain(processStep)
    expect(formatted).not.toContain(oldUpstreamOutput)
    expect(formatted).not.toContain(oldCollaborationStep)
    expect(formatted).not.toContain(internalFailureDetail)
  })
})

function t(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
