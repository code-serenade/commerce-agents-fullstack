export type UrlGroup = "api" | "auth" | "file"

export function resolveUrl(url: string, _group: UrlGroup = "api"): string {
  const isAbsolute = /^https?:\/\//.test(url)
  if (isAbsolute) return url
  return url
}
