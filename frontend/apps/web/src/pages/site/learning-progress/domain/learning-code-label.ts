import type { TFunction } from "i18next"

export function translateLearningCode(
  t: TFunction,
  prefix: string,
  code: string
) {
  const key = `${prefix}.${code}`
  const label = t(key)
  return label === key ? code : label
}
