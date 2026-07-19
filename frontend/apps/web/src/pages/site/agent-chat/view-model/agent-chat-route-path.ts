export function isAgentChatPath(pathname: string) {
  return (
    pathname === "/" ||
    pathname === "/agent" ||
    pathname === "/commerce" ||
    pathname.startsWith("/commerce/")
  )
}
