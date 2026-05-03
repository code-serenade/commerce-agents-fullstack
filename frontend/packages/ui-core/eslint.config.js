import { createReactPackageEslintConfig } from "@workspace/eslint-config/react-package"

export default createReactPackageEslintConfig({
  tsconfigRootDir: import.meta.dirname,
  ignores: ["dist", "src/components/**"],
})
