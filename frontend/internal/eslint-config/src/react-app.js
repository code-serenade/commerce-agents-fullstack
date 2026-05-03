import js from "@eslint/js"
import { fileURLToPath } from "node:url"
import globals from "globals"
import betterTailwindcss from "eslint-plugin-better-tailwindcss"
import reactHooks from "eslint-plugin-react-hooks"
import reactRefresh from "eslint-plugin-react-refresh"
import tseslint from "typescript-eslint"

const tailwindEntryPoint = fileURLToPath(
  new URL("../../../packages/ui-core/src/styles/globals.css", import.meta.url)
)

export function createReactAppEslintConfig({
  tsconfigRootDir,
  ignores = ["dist"],
  allowAny = true,
} = {}) {
  return tseslint.config(
    { ignores },
    {
      extends: [js.configs.recommended, ...tseslint.configs.recommended],
      files: ["**/*.{ts,tsx}"],
      languageOptions: {
        ecmaVersion: 2020,
        globals: globals.browser,
        parserOptions: {
          projectService: true,
          tsconfigRootDir,
        },
      },
      plugins: {
        "better-tailwindcss": betterTailwindcss,
        "react-hooks": reactHooks,
        "react-refresh": reactRefresh,
      },
      rules: {
        ...reactHooks.configs.recommended.rules,
        "better-tailwindcss/enforce-canonical-classes": "error",
        "react-refresh/only-export-components": [
          "warn",
          { allowConstantExport: true },
        ],
        ...(allowAny ? { "@typescript-eslint/no-explicit-any": "off" } : {}),
      },
      settings: {
        "better-tailwindcss": {
          entryPoint: tailwindEntryPoint,
        },
        "eslint-plugin-better-tailwindcss": {
          entryPoint: tailwindEntryPoint,
        },
      },
    }
  )
}
