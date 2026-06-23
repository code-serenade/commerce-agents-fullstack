import { describe, expect, it } from "vitest"
import { formatAgentUiMessage } from "./agent-chat-message-mapping"

const copy = {
  boundaryNotice: "boundary notice",
  missingStructuredOutput: "missing structured output",
  streamTimedOut: "stream timed out",
}

describe("agent answer sunscreen repeated summary fallback", () => {
  it("removes repeated sunscreen summary tail after a complete visible answer", () => {
    const heading = textFromCodes([
      20808, 32473, 20320, 29616, 25104, 30340, 21487, 29992, 32467,
      35770,
    ])
    const priorityHeading = textFromCodes([
      26368, 35813, 20248, 20808, 25913, 30340, 22320, 26041,
    ])
    const priorityBody = textFromCodes([
      29616, 22312, 20320, 21482, 26377, 26631, 39064, 12289, 20215,
      26684, 12289, 20027, 22270, 19977, 20010, 35201, 32032, 65292,
      28857, 20987, 29575, 20302, 39318, 20808, 35201, 20808, 25490,
      26597, 26631, 39064, 26159, 19981, 26159, 27809, 36393, 29992,
      25143, 25628, 38450, 26194, 34915, 26368, 24120, 29992, 30340,
      20851, 38190, 35789,
    ])
    const titleHeading = textFromCodes([
      21442, 32771, 33258, 28982, 26631, 39064,
    ])
    const title = textFromCodes([
      20912, 19997, 20937, 24863, 38450, 26194, 34915, 22899, 50, 48,
      50, 53, 26032, 27454, 38450, 32043, 22806, 32447, 85, 80, 70,
      53, 48, 43, 32, 22799, 23395, 36879, 27668, 34180, 27454,
    ])
    const dataHeading = textFromCodes([
      35201, 35760, 24405, 30340, 23383, 27573, 21644, 35745, 31639,
      26041, 24335,
    ])
    const sampleBoundary = textFromCodes([
      26679, 26412, 36793, 30028,
    ])
    const currentLimitHeading = textFromCodes([
      29616, 22312, 30340, 23616, 38480, 21644, 19979, 19968, 27493,
      32451, 25163, 26041, 27861,
    ])
    const currentLimitBody = textFromCodes([
      22240, 20026, 29616, 22312, 30475, 19981, 21040, 20320, 21407,
      26469, 30340, 26631, 39064, 12289, 20855, 20307, 23450, 20215,
      21644, 20027, 22270, 20869, 23481, 65292, 20320, 19979, 27425,
      21487, 20197, 25226, 36825, 19977, 20010, 20869, 23481, 37117,
      21457, 36807, 26469,
    ])
    const repeatedDataGap = textFromCodes([
      24403, 21069, 26080, 20855, 20307, 21830, 21697, 30340, 26631,
      39064, 12289, 20027, 22270, 12289, 20215, 26684, 32454, 33410,
      65292, 20063, 26080, 26333, 20809, 28857, 20987, 31561, 27969,
      37327, 25968, 25454, 65292, 21482, 33021, 22522, 20110, 36890,
      29992, 38450, 26194, 34915, 36816, 33829, 36923, 36753, 32473,
      20986, 20248, 21270, 26041, 21521,
    ])
    const repeatedPriority = textFromCodes([
      28857, 20987, 29575, 30340, 26680, 24515, 24433, 21709, 35201,
      32032, 20248, 20808, 32423, 20026, 65306, 26631, 39064, 20851,
      38190, 35789, 21305, 37197, 24230, 62, 20027, 22270, 21334,
      28857, 28165, 26224, 24230, 62, 20215, 26684, 31454, 20105,
      21147,
    ])
    const repeatedNoFabrication = textFromCodes([
      22914, 26524, 27809, 26377, 30495, 23454, 28431, 26007, 21644,
      28192, 36947, 25968, 25454, 65292, 21482, 33021, 36755, 20986,
      21028, 26029, 26694, 26550, 65292, 19981, 33021, 32534, 36896,
      36716, 21270, 29575, 25110, 34892, 19994, 25490, 21517,
    ])
    const repeatedNext = textFromCodes([
      20248, 20808, 20248, 21270, 26631, 39064, 65292, 34917, 20805,
      38450, 26194, 34915, 26680, 24515, 25628, 32034, 20851, 38190,
      35789,
    ])
    const repeatedSample = textFromCodes([
      21518, 32493, 20462, 25913, 35201, 32032, 21518, 31215, 32047,
      33267, 23569, 49, 48, 48, 48, 27425, 26333, 20809, 20877,
      35780, 20272, 25928, 26524,
    ])
    const repeatedSupplement = textFromCodes([
      21487, 34917, 20805, 21407, 26631, 39064, 12289, 20027, 22270,
      12289, 20215, 26684, 21644, 21518, 21488, 27969, 37327, 25968,
      25454, 33719, 24471, 26356, 31934, 20934, 30340, 20248, 21270,
      26041, 26696,
    ])
    const raw = [
      `## ${heading}`,
      `## ${priorityHeading}`,
      priorityBody,
      `## ${titleHeading}`,
      `「${title}」`,
      `## ${dataHeading}`,
      `${sampleBoundary}：${textFromCodes([20320, 27599, 27425, 25913, 23436, 26631, 39064, 21518, 33267, 23569, 25910, 38598, 26333, 20809, 25968, 25454])}`,
      `## ${currentLimitHeading}`,
      currentLimitBody,
      repeatedDataGap,
      repeatedPriority,
      repeatedNoFabrication,
      repeatedNext,
      repeatedSample,
      repeatedSupplement,
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).toContain(priorityHeading)
    expect(formatted).toContain(title)
    expect(formatted).toContain(dataHeading)
    expect(formatted).toContain(currentLimitHeading)
    expect(formatted).not.toContain(repeatedDataGap)
    expect(formatted).not.toContain(repeatedPriority)
    expect(formatted).not.toContain(repeatedNoFabrication)
    expect(formatted).not.toContain(repeatedNext)
    expect(formatted).not.toContain(repeatedSample)
    expect(formatted).not.toContain(repeatedSupplement)
  })

  it("keeps supplement questions while removing repeated sunscreen summary bullets", () => {
    const executionPlan = textFromCodes([25191, 34892, 26041, 26696])
    const riskBoundary = textFromCodes([39118, 38505, 36793, 30028])
    const title = textFromCodes([
      85, 80, 70, 53, 48, 43, 20937, 24863, 38450, 26194, 34915, 22899,
      22799, 23395, 38450, 32043, 22806, 32447,
    ])
    const titleHeading = textFromCodes([
      21442, 32771, 33258, 28982, 26631, 39064,
    ])
    const supplementHeading = textFromCodes([
      24453, 34917, 20805, 20449, 24687, 65288, 26041, 20415, 21518,
      32493, 26356, 31934, 20934, 20248, 21270, 65289,
    ])
    const questionOne = textFromCodes([
      20320, 24403, 21069, 22312, 29992, 30340, 26087, 26631, 39064,
      12289, 23450, 20215, 12289, 20027, 22270, 26680, 24515, 23637,
      31034, 30340, 20869, 23481, 26159, 20160, 20040, 65311,
    ])
    const questionTwo = textFromCodes([
      36825, 27454, 38450, 26194, 34915, 26368, 36817, 55, 22825,
      30340, 30340, 26333, 20809, 37327, 12289, 28857, 20987, 37327,
      20855, 20307, 26159, 22810, 23569, 65311,
    ])
    const repeatedMissing = textFromCodes([
      24403, 21069, 32570, 23569, 20855, 20307, 26631, 39064, 12289,
      23450, 20215, 12289, 20027, 22270, 20869, 23481, 21450, 21382,
      21490, 26333, 20809, 28857, 20987, 25968, 25454, 65292, 20165,
      33021, 22522, 20110, 38450, 26194, 34915, 21697, 31867, 36890,
      29992, 27969, 37327, 36923, 36753, 32473, 20986, 20248, 21270,
      26041, 21521,
    ])
    const repeatedPriority = textFromCodes([
      28857, 20987, 29575, 20302, 30340, 26680, 24515, 21487, 33021,
      24615, 20248, 20808, 32423, 65306, 26631, 39064, 26410, 35206,
      30422, 26680, 24515, 25628, 32034, 38656, 27714, 62, 20027,
      22270, 21334, 28857, 19981, 28165, 26224, 62, 20215, 26684,
      20559, 31163, 21516, 21697, 31867, 24120, 35268, 21306, 38388,
    ])
    const repeatedSearch = textFromCodes([
      38450, 26194, 34915, 31867, 30446, 29992, 25143, 39640, 39057,
      25628, 32034, 35789, 21253, 21547, 38450, 26194, 25351, 25968,
      12289, 38754, 26009, 12289, 36866, 29992, 22330, 26223, 12289,
      29256, 22411, 12289, 23395, 33410, 23646, 24615,
    ])
    const repeatedAction = textFromCodes([
      20248, 20808, 35843, 25972, 26631, 39064, 65292, 35206, 30422,
      38450, 26194, 34915, 26680, 24515, 25628, 32034, 35789, 21644,
      29992, 25143, 20851, 24515, 30340, 21334, 28857,
    ])
    const repeatedRisk = textFromCodes([
      21516, 26102, 20462, 25913, 26631, 39064, 12289, 20027, 22270,
      12289, 20215, 26684, 22810, 20010, 21464, 37327, 65292, 26080,
      27861, 21028, 26029, 21738, 20010, 35843, 25972, 24433, 21709,
      20102, 28857, 20987, 29575,
    ])
    const raw = [
      `## ${executionPlan}`,
      `| ${textFromCodes([20107, 39033])} | ${textFromCodes([21160, 20316])} |`,
      `| ${textFromCodes([26631, 39064, 35843, 25972])} | ${titleHeading}：${title} |`,
      `## ${riskBoundary}`,
      textFromCodes([
        26412, 36718, 21028, 26029, 22522, 20110, 36890, 29992, 21697,
        31867, 36923, 36753, 65292, 20379, 20320, 20808, 27979, 35797,
        21442, 32771,
      ]),
      `- ${supplementHeading}`,
      `- ${questionOne}`,
      `- ${questionTwo}`,
      `- ${repeatedMissing}`,
      `- ${repeatedPriority}`,
      `- ${repeatedSearch}`,
      `- ${repeatedAction}`,
      `- ${repeatedRisk}`,
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).toContain(title)
    expect(formatted).toContain(supplementHeading)
    expect(formatted).toContain(questionOne)
    expect(formatted).toContain(questionTwo)
    expect(formatted).not.toContain(repeatedMissing)
    expect(formatted).not.toContain(repeatedPriority)
    expect(formatted).not.toContain(repeatedSearch)
    expect(formatted).not.toContain(repeatedAction)
    expect(formatted).not.toContain(repeatedRisk)
  })
})

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}
