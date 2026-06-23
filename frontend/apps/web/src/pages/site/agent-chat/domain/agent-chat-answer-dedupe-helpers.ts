export function normalizeSectionHeading(line: string) {
  return line
    .trim()
    .replace(/^[-*•]\s*/, "")
    .replace(/^#+\s*/, "")
    .replace(/\s+/g, "")
    .replace(/[：:。?？]+$/g, "")
}


export function includesAll(value: string, termsToFind: string[]) {
  return termsToFind.every((term) => value.includes(term))
}


export function stripListPrefix(line: string) {
  return line.trim().replace(/^[-*•]\s*/, "")
}
