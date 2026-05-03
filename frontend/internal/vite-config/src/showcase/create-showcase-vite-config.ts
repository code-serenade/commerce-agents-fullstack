import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vitest/config"
import { createWorkspaceAliases } from "../react-app/aliases.ts"
import { RESOLVE_EXTENSIONS } from "../react-app/constants.ts"
import type { CreateShowcaseViteConfigOptions } from "./types.ts"

export function createShowcaseViteConfig({
  appDir,
  testSetupFiles = ["./src/test/setup.ts"],
}: CreateShowcaseViteConfigOptions) {
  return defineConfig({
    plugins: [react(), tailwindcss()],
    resolve: {
      extensions: RESOLVE_EXTENSIONS,
      alias: createWorkspaceAliases(appDir),
    },
    test: {
      environment: "jsdom",
      globals: true,
      setupFiles: testSetupFiles,
      css: true,
      server: {
        deps: {
          inline: [],
        },
      },
    },
  })
}
