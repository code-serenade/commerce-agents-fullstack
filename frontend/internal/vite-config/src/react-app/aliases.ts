import path from "node:path"

export function createWorkspaceAliases(appDir: string) {
  return [
    {
      find: /^@$/,
      replacement: path.resolve(appDir, "src"),
    },
    {
      find: /^@workspace\/ui-components$/,
      replacement: path.resolve(
        appDir,
        "../../packages/ui-components/src/index.ts"
      ),
    },
    {
      find: /^@workspace\/app-kit$/,
      replacement: path.resolve(appDir, "../../packages/app-kit/src/index.ts"),
    },
    {
      find: /^@workspace\/services$/,
      replacement: path.resolve(appDir, "../../packages/services/src/index.ts"),
    },
    {
      find: /^@workspace\/services\/(.+)$/,
      replacement: path.resolve(appDir, "../../packages/services/src/$1"),
    },
    {
      find: /^@workspace\/locales$/,
      replacement: path.resolve(appDir, "../../packages/locales/src/index.ts"),
    },
    {
      find: /^@workspace\/locales\/(.+)$/,
      replacement: `${path.resolve(appDir, "../../packages/locales/src")}/$1.ts`,
    },
    {
      find: /^react-router$/,
      replacement: path.resolve(appDir, "node_modules/react-router"),
    },
    {
      find: "@/",
      replacement: `${path.resolve(appDir, "src")}/`,
    },
  ]
}
