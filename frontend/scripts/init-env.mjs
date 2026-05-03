import fs from "node:fs/promises"
import path from "node:path"
import { fileURLToPath } from "node:url"

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const workspaceRoot = path.resolve(scriptDir, "..")
const examplePath = path.join(workspaceRoot, "env.shared.example")

const modes = ["development", "production"]
const rootKeys = ["VITE_AUTH_PROXY", "VITE_FILE_PROXY"]
const appKeys = ["VITE_API_PROXY"]
const apps = ["admin", "web"]

function parseExample(source) {
  const sections = new Map()
  let currentMode = null

  for (const rawLine of source.split(/\r?\n/)) {
    const line = rawLine.trim()

    if (!line) continue

    if (line.startsWith("#")) {
      const mode = line.slice(1).trim().toLowerCase()
      if (modes.includes(mode)) {
        currentMode = mode
        sections.set(currentMode, new Map())
      }
      continue
    }

    if (!currentMode) {
      throw new Error(`env.shared.example contains a value before a mode header: ${line}`)
    }

    const separatorIndex = line.indexOf("=")
    if (separatorIndex === -1) {
      throw new Error(`Invalid env line in ${currentMode}: ${line}`)
    }

    const key = line.slice(0, separatorIndex).trim()
    const value = line.slice(separatorIndex + 1).trim()
    sections.get(currentMode).set(key, value)
  }

  return sections
}

function pickEnv(sections, mode, keys) {
  const values = sections.get(mode)

  if (!values) {
    throw new Error(`Missing # ${mode} section in env.shared.example`)
  }

  return keys.map((key) => {
    const value = values.get(key)

    if (value === undefined) {
      throw new Error(`Missing ${key} in # ${mode} section`)
    }

    return `${key}=${value}`
  })
}

async function writeEnvFile(filePath, lines) {
  await fs.mkdir(path.dirname(filePath), { recursive: true })
  await fs.writeFile(filePath, `${lines.join("\n")}\n`)
  console.log(`created ${path.relative(workspaceRoot, filePath)}`)
}

const source = await fs.readFile(examplePath, "utf8")
const sections = parseExample(source)

for (const mode of modes) {
  await writeEnvFile(
    path.join(workspaceRoot, `.env.${mode}`),
    pickEnv(sections, mode, rootKeys)
  )

  for (const app of apps) {
    await writeEnvFile(
      path.join(workspaceRoot, "apps", app, `.env.${mode}`),
      pickEnv(sections, mode, appKeys)
    )
  }
}
