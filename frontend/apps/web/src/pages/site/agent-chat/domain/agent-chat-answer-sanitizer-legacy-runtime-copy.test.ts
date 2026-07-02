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

  it("rewrites old persisted evidence labels into task process wording", () => {
    const legacyViewEvidence = t(26597, 30475, 21028, 26029, 20381, 25454)
    const legacyRoundEvidence = t(26412, 36718, 21028, 26029, 20381, 25454)
    const legacyReportContext = t(25253, 21578, 35821, 22659)
    const legacyJudgmentTrace = t(21028, 26029, 33033, 32476)
    const taskProcess = t(20219, 21153, 36807, 31243)
    const viewTaskProcess = t(26597, 30475, 20219, 21153, 36807, 31243)
    const roundTaskProcess = t(26412, 36718, 20219, 21153, 36807, 31243)
    const reportSuggestion = t(22797, 30424, 25253, 21578, 24314, 35758)
    const processSteps = t(22788, 29702, 27493, 39588)
    const raw = [
      `${legacyViewEvidence}: ${t(24050, 23436, 25104)}`,
      `## ${legacyRoundEvidence}`,
      `${legacyReportContext}: ${t(24314, 35758)}`,
      `${legacyJudgmentTrace}: ${t(20808, 30475, 36164, 26009)}`,
      `## ${t(21028, 26029, 20381, 25454)}`,
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).toContain(viewTaskProcess)
    expect(formatted).toContain(roundTaskProcess)
    expect(formatted).toContain(reportSuggestion)
    expect(formatted).toContain(processSteps)
    expect(formatted).toContain(`## ${taskProcess}`)
    expect(formatted).not.toContain(legacyViewEvidence)
    expect(formatted).not.toContain(legacyRoundEvidence)
    expect(formatted).not.toContain(legacyReportContext)
    expect(formatted).not.toContain(legacyJudgmentTrace)
  })

  it("rewrites old answer headings and evidence phrases into conclusion and reference wording", () => {
    const judgment = t(21028, 26029)
    const taskMaterial = t(20219, 21153, 36164, 26009)
    const conclusion = t(32467, 35770)
    const referenceMaterial = t(21442, 32771, 26448, 26009)
    const processEvidence = t(36807, 31243, 12289, 20381, 25454)
    const riskEvidence = t(39118, 38505, 12289, 20381, 25454)
    const processReference = t(36807, 31243, 12289, 21442, 32771, 26448, 26009)
    const riskReference = t(39118, 38505, 12289, 21442, 32771, 26448, 26009)
    const raw = [
      `## ${judgment}`,
      t(20808, 20889, 35797, 38382, 39064, 21487, 20197, 22788, 29702),
      `## ${taskMaterial}`,
      `${processEvidence}${t(38656, 35201, 32487, 32493, 25972, 29702)}`,
      `${riskEvidence}${t(38656, 35201, 20998, 24320, 35828, 26126)}`,
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).toContain(`## ${conclusion}`)
    expect(formatted).toContain(`## ${referenceMaterial}`)
    expect(formatted).toContain(processReference)
    expect(formatted).toContain(riskReference)
    expect(formatted).not.toContain(`## ${judgment}`)
    expect(formatted).not.toContain(`## ${taskMaterial}`)
    expect(formatted).not.toContain(processEvidence)
    expect(formatted).not.toContain(riskEvidence)
  })
})

function t(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
