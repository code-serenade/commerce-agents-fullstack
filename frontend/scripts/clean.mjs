import { execSync } from "node:child_process"
import { existsSync, readdirSync, rmSync, statSync } from "node:fs"
import path from "node:path"
import { fileURLToPath } from "node:url"

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..")
const removableDirs = new Set([
  "node_modules",
  "dist",
  "build",
  ".turbo",
  ".cache",
  ".vite",
  "coverage",
])

const removed = []
const removedTsBuildInfo = []

function removeDir(target) {
  rmSync(target, { recursive: true, force: true })
  removed.push(path.relative(rootDir, target) || ".")
}

function walk(dir) {
  const entries = readdirSync(dir, { withFileTypes: true })

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name)

    if (entry.isDirectory()) {
      if (removableDirs.has(entry.name)) {
        removeDir(fullPath)
        continue
      }

      if (entry.name === ".git") {
        continue
      }

      walk(fullPath)
      continue
    }

    if (entry.isFile() && entry.name.endsWith(".tsbuildinfo")) {
      rmSync(fullPath, { force: true })
      removedTsBuildInfo.push(path.relative(rootDir, fullPath))
    }
  }
}

walk(rootDir)

console.log(`[clean] removed ${removed.length} directories`)
for (const item of removed) {
  console.log(` - ${item}`)
}

if (removedTsBuildInfo.length > 0) {
  console.log(`[clean] removed ${removedTsBuildInfo.length} tsbuildinfo files`)
  for (const item of removedTsBuildInfo) {
    console.log(` - ${item}`)
  }
}

if (existsSync(path.join(rootDir, "pnpm-lock.yaml"))) {
  console.log("[clean] running pnpm store prune...")
  execSync("pnpm store prune", {
    cwd: rootDir,
    stdio: "inherit",
  })
}

console.log("[clean] done")
