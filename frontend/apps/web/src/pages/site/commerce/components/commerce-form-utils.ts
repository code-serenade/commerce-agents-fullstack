export interface NumericInputOptions {
  decimalScale?: number
  nonNegative?: boolean
}

export function normalizeNumericInput(
  value: string,
  { decimalScale, nonNegative = false }: NumericInputOptions = {}
) {
  if (decimalScale == null) {
    return nonNegative ? value.replace(/^-/, "") : value
  }

  const sign = value.startsWith("-") && !nonNegative ? "-" : ""
  const unsignedValue = value.replace(/^-/, "")
  const [integerPart = "", ...fractionParts] = unsignedValue.split(".")
  const integerDigits = integerPart.replace(/\D/g, "")
  const hasDecimalPoint = unsignedValue.includes(".")
  const fractionDigits = fractionParts
    .join("")
    .replace(/\D/g, "")
    .slice(0, decimalScale)

  if (!integerDigits && !hasDecimalPoint) return ""

  return `${sign}${integerDigits || "0"}${
    hasDecimalPoint ? `.${fractionDigits}` : ""
  }`
}
