const MAX_BARE_HEADING_LENGTH = 32

export function isReadableBareHeading(line: string) {
  const text = line.trim()
  if (
    Array.from(text).length < 2 ||
    Array.from(text).length > MAX_BARE_HEADING_LENGTH ||
    !/[\u4e00-\u9fa5]/.test(text) ||
    text.includes("http") ||
    /[。！？!?；;，,]$/.test(text)
  ) {
    return false
  }

  if (!Array.from(text).every((char) => isBareHeadingTextChar(char))) {
    return false
  }

  const normalized = text.replace(/[（(].*[）)]$/g, "")
  return (
    BARE_HEADING_TERMS.includes(text) ||
    BARE_HEADING_TERMS.includes(normalized) ||
    BARE_HEADING_SUFFIXES.some((suffix) => normalized.endsWith(suffix))
  )
}

const BARE_HEADING_TERMS = [
  [0x54c1, 0x7c7b],
  [0x6807, 0x9898, 0x6a21, 0x677f],
].map((codes) => String.fromCodePoint(...codes))

const BARE_HEADING_SUFFIXES = [
  [0x5bf9, 0x8c61],
  [0x9009, 0x62e9],
  [0x7d20, 0x6750],
  [0x89c4, 0x8303],
  [0x9884, 0x7b97],
  [0x53e3, 0x5f84],
  [0x63d0, 0x793a],
  [0x4fe1, 0x606f],
  [0x6a21, 0x677f],
  [0x98ce, 0x9669],
  [0x95ee, 0x9898],
  [0x8bdd, 0x672f],
  [0x6e05, 0x5355],
  [0x6b65, 0x9aa4],
].map((codes) => String.fromCodePoint(...codes))

function isBareHeadingTextChar(char: string) {
  const codePoint = char.codePointAt(0) ?? 0
  return (
    /[\u4e00-\u9fa5A-Za-z0-9]/.test(char) ||
    codePoint === 0x002f ||
    codePoint === 0xff08 ||
    codePoint === 0xff09 ||
    codePoint === 0x0028 ||
    codePoint === 0x0029 ||
    codePoint === 0x3001 ||
    codePoint === 0x00b7 ||
    /\s/.test(char)
  )
}
