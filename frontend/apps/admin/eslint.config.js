import { createReactAppEslintConfig } from "@workspace/eslint-config/react-app"

export default createReactAppEslintConfig({
  tsconfigRootDir: import.meta.dirname,
})
