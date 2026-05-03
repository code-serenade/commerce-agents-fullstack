import type { ToasterProps } from "sonner"

import type { BaseMode } from "../../lib/component-mode"

export type SonnerClassNameMode = "merge" | "replace"

export type SonnerClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type SonnerToasterProps = ToasterProps & {
  mode?: BaseMode
  classNameMode?: SonnerClassNameMode
  classResolver?: SonnerClassResolver
  iconClassName?: string
  iconClassNameMode?: SonnerClassNameMode
  iconClassResolver?: SonnerClassResolver
  loadingIconClassName?: string
  loadingIconClassNameMode?: SonnerClassNameMode
  loadingIconClassResolver?: SonnerClassResolver
  toastClassName?: string
  toastClassNameMode?: SonnerClassNameMode
  toastClassResolver?: SonnerClassResolver
}
