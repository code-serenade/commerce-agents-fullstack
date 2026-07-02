const JUDGMENT_EVIDENCE = cjk(21028, 26029, 20381, 25454)
const JUDGMENT = cjk(21028, 26029)
const CONCLUSION = cjk(32467, 35770)
const TASK_PROCESS = cjk(20219, 21153, 36807, 31243)
const TASK_MATERIAL = cjk(20219, 21153, 36164, 26009)
const REFERENCE_MATERIAL = cjk(21442, 32771, 26448, 26009)
const REVIEW_CONCLUSION = cjk(22797, 26680, 32467, 35770)
const EVALUATION = cjk(35780, 20272)

const LEGACY_PROCESS_COPY_REPLACEMENTS = [
  [
    cjk(26597, 30475, 21028, 26029, 20381, 25454),
    cjk(26597, 30475, 20219, 21153, 36807, 31243),
  ],
  [
    cjk(23637, 24320, 21028, 26029, 20381, 25454),
    cjk(23637, 24320, 20219, 21153, 36807, 31243),
  ],
  [
    cjk(25910, 36215, 21028, 26029, 20381, 25454),
    cjk(25910, 36215, 20219, 21153, 36807, 31243),
  ],
  [
    cjk(25240, 21472, 21028, 26029, 20381, 25454),
    cjk(25910, 36215, 20219, 21153, 36807, 31243),
  ],
  [
    cjk(26412, 36718, 21028, 26029, 20381, 25454),
    cjk(26412, 36718, 20219, 21153, 36807, 31243),
  ],
  [
    cjk(24403, 21069, 21028, 26029, 20381, 25454),
    cjk(24403, 21069, 20219, 21153, 36807, 31243),
  ],
  [cjk(21028, 26029, 33033, 32476), cjk(22788, 29702, 27493, 39588)],
  [
    cjk(25253, 21578, 35821, 22659),
    cjk(22797, 30424, 25253, 21578, 24314, 35758),
  ],
  [
    cjk(21028, 26029, 20381, 25454, 38754, 26495),
    cjk(20219, 21153, 36807, 31243, 38754, 26495),
  ],
  [
    cjk(20381, 25454, 38754, 26495),
    cjk(20219, 21153, 36807, 31243, 38754, 26495),
  ],
  [cjk(36164, 26009, 20381, 25454), cjk(20219, 21153, 36164, 26009)],
  [cjk(20316, 20026, 36873, 21697, 21028, 26029, 20381, 25454), cjk(20316, 20026, 36873, 21697, 21442, 32771, 26448, 26009)],
  [cjk(36807, 31243, 12289, 20381, 25454), cjk(36807, 31243, 12289, 21442, 32771, 26448, 26009)],
  [cjk(39118, 38505, 12289, 20381, 25454), cjk(39118, 38505, 12289, 21442, 32771, 26448, 26009)],
  [cjk(25972, 29702, 21028, 26029), cjk(25972, 29702, 32467, 35770)],
  [cjk(24037, 31243, 21028, 26029), cjk(24037, 31243, 35780, 20272)],
  [cjk(21487, 21028, 26029, 32447, 32034), cjk(21487, 35780, 20272, 32447, 32034)],
  [cjk(25226, 21028, 26029, 33853, 21040), cjk(25226, 32467, 35770, 33853, 21040)],
  [cjk(24050, 26377, 36164, 26009, 21028, 26029), cjk(24050, 26377, 36164, 26009, 32473, 20986, 32467, 35770)],
  [cjk(22797, 26680, 21028, 26029), REVIEW_CONCLUSION],
  [cjk(37325, 26032, 21028, 26029), cjk(37325, 26032, 35780, 20272)],
  [cjk(25165, 33021, 21028, 26029), cjk(25165, 33021, 35780, 20272)],
  [cjk(21021, 27493, 36873, 21697, 21028, 26029), cjk(21021, 27493, 36873, 21697, 32467, 35770)],
  [cjk(36716, 21270, 29942, 39048, 21028, 26029), cjk(36716, 21270, 29942, 39048, 35780, 20272)],
  [cjk(24110, 21161, 23398, 29983, 21028, 26029), cjk(24110, 21161, 23398, 29983, 30830, 35748)],
  [cjk(26681, 25454, 38382, 39064, 21028, 26029), cjk(26681, 25454, 38382, 39064, 35782, 21035)],
] as const

export function normalizeLegacyProcessCopyLine(line: string) {
  const normalizedLabel = normalizeLegacyProcessLabel(line)
  if (normalizedLabel !== null) return normalizedLabel

  return LEGACY_PROCESS_COPY_REPLACEMENTS.reduce(
    (current, [from, to]) => current.replaceAll(from, to),
    line
  )
}

function normalizeLegacyProcessLabel(line: string) {
  const match = line.match(/^(\s*(?:#{1,6}\s*)?(?:[-*•]\s*)?)(.*)$/)
  if (!match) return null

  const prefix = match[1] ?? ""
  const body = (match[2] ?? "").trim()
  const suffixMatch = body.match(/^(.+?)([:：]?)$/)
  const label = (suffixMatch?.[1] ?? body).trim()
  const suffix = suffixMatch?.[2] ?? ""

  if (label === JUDGMENT) return `${prefix}${CONCLUSION}${suffix}`
  if (label === REVIEW_CONCLUSION) return `${prefix}${CONCLUSION}${suffix}`
  if (label === EVALUATION) return `${prefix}${CONCLUSION}${suffix}`
  if (label === TASK_MATERIAL) return `${prefix}${REFERENCE_MATERIAL}${suffix}`
  if (label !== JUDGMENT_EVIDENCE) return null

  return `${prefix}${TASK_PROCESS}${suffix}`
}

function cjk(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
