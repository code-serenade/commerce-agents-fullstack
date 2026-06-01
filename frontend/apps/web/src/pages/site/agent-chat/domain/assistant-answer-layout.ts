const BRACKET_OPEN = String.fromCharCode(0x3010)
const BRACKET_CLOSE = String.fromCharCode(0x3011)
const EXAMPLE_MARKER = String.fromCharCode(0x5982)
const FULL_WIDTH_COLON = String.fromCharCode(0xff1a)
const HEADING_PUNCTUATION = new Set([
  0x0021,
  0x002c,
  0x003b,
  0x003f,
  0xff01,
  0xff0c,
  0xff1b,
  0xff1f,
  0x3002,
])
const SENTENCE_BOUNDARY = new Set([
  0x0021,
  0x003b,
  0x003f,
  0xff01,
  0xff1b,
  0xff1f,
  0x3002,
])
const SECTION_SUFFIXES = [
  [0x7ed3, 0x8bba],
  [0x5224, 0x65ad],
  [0x8bf4, 0x660e],
  [0x4f9d, 0x636e],
  [0x52a8, 0x4f5c],
  [0x5efa, 0x8bae],
  [0x8fb9, 0x754c],
  [0x98ce, 0x9669],
  [0x95ee, 0x9898],
  [0x6e05, 0x5355],
  [0x8ba1, 0x5212],
  [0x7ec3, 0x4e60],
  [0x8bad, 0x7ec3],
  [0x5b9e, 0x8bad],
  [0x65b9, 0x6848],
  [0x7b56, 0x7565],
  [0x6b65, 0x9aa4],
  [0x91cd, 0x70b9],
  [0x73af, 0x8282],
  [0x90e8, 0x5206],
  [0x65b9, 0x5411],
  [0x89d2, 0x5ea6],
  [0x8d44, 0x6599],
  [0x6570, 0x636e],
  [0x539f, 0x56e0],
  [0x9650, 0x5236],
  [0x6838, 0x9a8c],
  [0x4e0b, 0x4e00, 0x6b65],
  [0x9002, 0x5408],
  [0x4e0d, 0x9002, 0x5408],
].map((codes) => String.fromCodePoint(...codes))

export function normalizeAnswerLayout(content: string) {
  return insertBreaksAroundInlineColonHeadings(
    insertBreaksBeforeInlineBracketHeadings(
      insertBreaksAfterInlineBracketHeadings(content)
    )
  ).replace(/\n{3,}/g, "\n\n")
}

export function readBracketHeadingText(line: string) {
  if (!line.startsWith(BRACKET_OPEN) || !line.endsWith(BRACKET_CLOSE)) {
    return null
  }
  const heading = line.slice(BRACKET_OPEN.length, -BRACKET_CLOSE.length)
  if (heading.length > 32 || heading.includes(BRACKET_CLOSE)) return null
  return isReadableBracketHeading(heading) ? heading : null
}

function insertBreaksBeforeInlineBracketHeadings(content: string) {
  let output = ""
  for (let index = 0; index < content.length; index += 1) {
    const char = content[index]
    if (
      char === BRACKET_OPEN &&
      output.length > 0 &&
      !output.endsWith("\n") &&
      isBracketHeadingAt(content, index)
    ) {
      output = output.replace(/[ \t]+$/g, "")
      output += "\n\n"
    }
    output += char
  }
  return output
}

function insertBreaksAfterInlineBracketHeadings(content: string) {
  let output = ""
  for (let index = 0; index < content.length; index += 1) {
    const char = content[index]
    output += char
    if (
      char === BRACKET_CLOSE &&
      isBracketHeadingEndingAt(output, output.length - 1) &&
      content[index + 1] &&
      content[index + 1] !== "\n"
    ) {
      output += "\n\n"
    }
  }
  return output
}

function isBracketHeadingAt(content: string, index: number) {
  const end = content.indexOf(BRACKET_CLOSE, index + 1)
  if (end < 0 || end - index > 34) return false
  const heading = content.slice(index + 1, end)
  return isReadableBracketHeading(heading)
}

function isBracketHeadingEndingAt(content: string, endIndex: number) {
  const lineStart = Math.max(content.lastIndexOf("\n", endIndex) + 1, 0)
  const start = content.lastIndexOf(BRACKET_OPEN, endIndex)
  if (start < lineStart) return false
  const heading = content.slice(start + 1, endIndex)
  return isReadableBracketHeading(heading)
}

function isReadableBracketHeading(heading: string) {
  const text = heading.trim()
  return (
    text.length > 0 &&
    text.length <= 32 &&
    /[\u4e00-\u9fa5A-Za-z]/.test(text) &&
    !Array.from(text).some((char) =>
      HEADING_PUNCTUATION.has(char.codePointAt(0) ?? 0)
    ) &&
    !text.includes(EXAMPLE_MARKER) &&
    !text.includes("/")
  )
}

function insertBreaksAroundInlineColonHeadings(content: string) {
  const chars = Array.from(content)
  const breakBefore = Array.from({ length: chars.length }, () => false)
  const breakAfter = Array.from({ length: chars.length }, () => false)

  for (let index = 0; index < chars.length; index += 1) {
    if (!isHeadingColon(chars[index] ?? "")) continue
    const start = inlineColonHeadingStart(chars, index)
    if (start === null) continue
    const heading = chars.slice(start, index).join("").trim()
    if (!isReadableColonHeading(heading)) continue
    if (start > 0 && chars[start - 1] !== "\n") breakBefore[start] = true
    if (chars[index + 1] && chars[index + 1] !== "\n") breakAfter[index] = true
  }

  let output = ""
  chars.forEach((char, index) => {
    if (breakBefore[index]) output = withParagraphBreak(output)
    output += char
    if (breakAfter[index]) output = withParagraphBreak(output)
  })
  return output
}

function isHeadingColon(char: string) {
  return char === ":" || char === FULL_WIDTH_COLON
}

function inlineColonHeadingStart(chars: string[], colonIndex: number) {
  if (colonIndex <= 0) return null
  let start = colonIndex
  while (start > 0) {
    const previous = chars[start - 1] ?? ""
    if (previous === "\n" || isSentenceBoundary(previous)) break
    start -= 1
  }
  while (start < colonIndex && /\s/.test(chars[start] ?? "")) start += 1
  return start < colonIndex ? start : null
}

function isSentenceBoundary(char: string) {
  return SENTENCE_BOUNDARY.has(char.codePointAt(0) ?? 0)
}

function isReadableColonHeading(heading: string) {
  const text = heading.trim()
  return (
    text.length >= 2 &&
    text.length <= 28 &&
    /[\u4e00-\u9fa5]/.test(text) &&
    Array.from(text).every((char) => isHeadingTextChar(char) || /\s/.test(char)) &&
    !text.includes(EXAMPLE_MARKER) &&
    SECTION_SUFFIXES.some((suffix) => text.endsWith(suffix))
  )
}

function isHeadingTextChar(char: string) {
  const codePoint = char.codePointAt(0) ?? 0
  return (
    /[\u4e00-\u9fa5A-Za-z0-9]/.test(char) ||
    codePoint === 0x0025 ||
    codePoint === 0xff05 ||
    codePoint === 0x3001 ||
    codePoint === 0x00b7 ||
    codePoint === 0x002d ||
    codePoint === 0x005f
  )
}

function withParagraphBreak(value: string) {
  const output = value.replace(/[ \t]+$/g, "")
  if (!output || output.endsWith("\n\n")) return output
  if (output.endsWith("\n")) return `${output}\n`
  return `${output}\n\n`
}
