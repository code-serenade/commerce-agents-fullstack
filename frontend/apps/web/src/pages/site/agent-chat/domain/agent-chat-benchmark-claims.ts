const ASSERTIVE_BENCHMARK_TERMS = terms([
  [25972, 20307, 20559, 22909],
  [30495, 27491, 20559, 22909],
  [30456, 23545, 20248, 24322],
  [31526, 21512, 34892, 19994],
  [31526, 21512, 19968, 33324, 30452, 25773],
  [31526, 21512, 22810, 25968],
  [21305, 37197, 24230, 33391, 22909],
  [20248, 20110],
  [39640, 20110],
  [20302, 20110],
  [23646, 20110],
  [30053, 39640],
  [30053, 20302],
  [25509, 36817],
  [20559, 39640],
  [20559, 20302],
  [36739, 39640],
  [36739, 20302],
  [20013, 31561, 27700, 24179],
  [34920, 29616, 33391, 22909],
  [34920, 29616, 36739, 22909],
  [34920, 29616, 20248, 31168],
  [36716, 21270, 29575, 39640],
  [28857, 20987, 29575, 20302],
  [28857, 20987, 29575, 39640],
  [21152, 36141, 29575, 20302],
  [21152, 36141, 29575, 39640],
  [19979, 21333, 29575, 20302],
  [19979, 21333, 29575, 39640],
  [20184, 27454, 29575, 20302],
  [20184, 27454, 29575, 39640],
  [38459, 21147, 26497, 20302],
  [23384, 22312, 19968, 23450, 38459, 21147],
  [22788, 20110, 24120, 35265, 27700, 24179],
  [20581, 24247, 21306, 38388],
  [20581, 24247, 27700, 24179],
  [34892, 19994, 24120, 35268, 27700, 24179],
])

const BENCHMARK_TERMS = terms([
  [34892, 19994, 24179, 22343],
  [34892, 19994, 22343, 20540],
  [34892, 19994, 22522, 20934],
  [34892, 19994, 27700, 24179],
  [34892, 19994, 24120, 35268],
  [21516, 31867, 30452, 25773],
  [21516, 31867, 21830, 21697],
  [21516, 31867, 22330, 26223],
  [19968, 33324, 30452, 25773],
  [22810, 25968, 30005, 21830, 30452, 25773],
  [22810, 25968, 30452, 25773],
  [24179, 22343, 27700, 24179],
  [24120, 35265, 33539, 22260],
  [24120, 35265, 27700, 24179],
  [34892, 19994, 21306, 38388],
  [34892, 19994, 33539, 22260],
  [34892, 19994],
])

const BENCHMARK_GAP_TERMS = terms([
  [38656, 35201],
  [24453, 34917],
  [32570, 20047],
  [32570, 23569],
  [25910, 38598],
  [23545, 27604],
  [22797, 26680],
  [34917, 40784],
  [25509, 20837],
  [26410, 25509, 20837],
  [27809, 26377, 30495, 23454],
  [19981, 33021],
  [19981, 24471],
  [19981, 36755, 20986],
  [19981, 21487],
  [26080, 27861, 23458, 35266, 21028, 26029],
])

const SOURCE_EVIDENCE_TERMS = terms([
  [26469, 28304],
  [24341, 29992],
  [35777, 25454],
  [24050, 20445, 23384, 30495, 23454, 22806, 37096, 25628, 32034, 35777, 25454],
])

const TITLE_ADVICE_TERMS = terms([
  [26631, 39064],
  [26680, 24515, 38656, 27714, 35789],
  [21069, 32622],
  [25918, 22312, 26631, 39064, 26368, 21069, 38754],
  [29992, 25143, 25628],
])

const BENCHMARK_CLAIM_TERMS = terms([
  [31526, 21512],
  [22788, 20110],
  [23646, 20110],
  [24120, 35265, 27700, 24179],
  [24179, 22343, 27700, 24179],
  [24120, 35268, 27700, 24179],
  [20581, 24247, 27700, 24179],
  [20581, 24247, 21306, 38388],
  [21560, 24341, 21147, 39044, 26399],
  [33391, 22909],
  [20248, 31168],
  [27491, 24120],
  [21512, 29702],
])

const BENCHMARK_NUMBER_TERMS = terms([
  [34892, 19994],
  [22522, 20934],
  [22343, 20540],
  [24179, 22343],
  [24120, 35268],
  [24120, 35265],
  [21306, 38388],
  [33539, 22260],
  [27700, 24179],
])

export function isUnsupportedBenchmarkClaim(value: string) {
  const text = value.trim()
  if (!text) return false
  if (
    containsAnyTerm(text, SOURCE_EVIDENCE_TERMS) ||
    containsTitleAdvice(text) ||
    /search_evidence|https?:\/\//.test(text)
  ) {
    return false
  }
  if (containsAnyTerm(text, ASSERTIVE_BENCHMARK_TERMS)) return true
  if (!containsAnyTerm(text, BENCHMARK_TERMS)) return false

  const asksForBenchmark = containsAnyTerm(text, BENCHMARK_GAP_TERMS)
  if (asksForBenchmark && !containsBenchmarkNumberClaim(text)) return false
  if (containsAnyTerm(text, BENCHMARK_CLAIM_TERMS)) return true
  return containsBenchmarkNumberClaim(text)
}

function containsBenchmarkNumberClaim(value: string) {
  return BENCHMARK_NUMBER_TERMS.some((term) => {
    const index = value.indexOf(term)
    if (index < 0) return false
    const after = value.slice(index).split(/[，。；;,\n]/)[0] ?? ""
    const before = value.slice(0, index).split(/[，。；;,\n]/).at(-1) ?? ""
    return containsPercentOrRange(after) || containsPercentOrRange(before)
  })
}

function containsPercentOrRange(value: string) {
  if (/\d\s*(?:%|％|-|~|—)/.test(value)) return true
  const rangeMarkerIndex = value.indexOf(cjk(33267))
  return rangeMarkerIndex >= 0 && /\d\s*$/.test(value.slice(0, rangeMarkerIndex))
}

function containsAnyTerm(value: string, termsToFind: string[]) {
  return termsToFind.some((term) => value.includes(term))
}

function containsTitleAdvice(value: string) {
  return TITLE_ADVICE_TERMS.filter((term) => value.includes(term)).length >= 2
}

function terms(codePointGroups: number[][]) {
  return codePointGroups.map((codes) => cjk(...codes))
}

function cjk(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
