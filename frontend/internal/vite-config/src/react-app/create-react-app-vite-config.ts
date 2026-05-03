import path from "node:path"
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vitest/config"
import { loadEnv } from "vite"
import { createWorkspaceAliases } from "./aliases.ts"
import { createManualChunkName } from "./chunks.ts"
import { RESOLVE_EXTENSIONS } from "./constants.ts"
import { createServerProxy } from "./proxy.ts"
import type { CreateReactAppViteConfigOptions } from "./types.ts"

export function createReactAppViteConfig({
  appDir,
  rewriteApiPath,
  testSetupFiles = ["./src/test/setup.ts"],
}: CreateReactAppViteConfigOptions) {
  return defineConfig(({ mode }) => {
    const sharedEnvDir =
      process.env.FRONTEND_ENV_DIR ?? path.resolve(appDir, "../../")
    const sharedEnv = loadEnv(mode, sharedEnvDir, "")
    const appEnv = loadEnv(mode, appDir, "")
    const env = { ...sharedEnv, ...appEnv }

    return {
      envDir: appDir,
      plugins: [react(), tailwindcss()],
      resolve: {
        extensions: RESOLVE_EXTENSIONS,
        alias: createWorkspaceAliases(appDir),
      },
      build: {
        rollupOptions: {
          output: {
            manualChunks(id) {
              return createManualChunkName(id)
            },
          },
        },
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
      server: {
        proxy: createServerProxy(env, { rewriteApiPath }),
      },
    }
  })
}
