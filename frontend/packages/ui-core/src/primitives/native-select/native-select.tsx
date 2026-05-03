import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { ChevronDownIcon } from "../../lib/icon-slots"
import { nativeSelectClassNames } from "./native-select.styles"
import type {
  NativeSelectClassResolver,
  NativeSelectOptGroupProps,
  NativeSelectOptionProps,
  NativeSelectProps,
  NativeSelectSize,
} from "./native-select.types"

function resolveStyledNativeSelectClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: NativeSelectClassResolver
}) {
  if (classResolver) {
    return classResolver({
      defaultClassName,
      className,
    })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function NativeSelect({
  mode = DEFAULT_MODE,
  className,
  size = "default",
  classNameMode = "merge",
  classResolver,
  selectClassName,
  selectClassNameMode = "merge",
  selectClassResolver,
  iconClassName,
  iconClassNameMode = "merge",
  iconClassResolver,
  ...props
}: NativeSelectProps) {
  if (mode === "primitive") {
    return (
      <div className={className}>
        <select className={selectClassName} {...props} />
        <ChevronDownIcon className={iconClassName} aria-hidden="true" />
      </div>
    )
  }

  const resolvedSize = (size ?? "default") as NativeSelectSize

  return (
    <div
      className={resolveStyledNativeSelectClassName({
        className,
        defaultClassName: nativeSelectClassNames.slot0,
        classNameMode,
        classResolver,
      })}
      data-slot="native-select-wrapper"
      data-size={resolvedSize}
    >
      <select
        data-slot="native-select"
        data-size={resolvedSize}
        className={resolveStyledNativeSelectClassName({
          className: selectClassName,
          defaultClassName: nativeSelectClassNames.slot1,
          classNameMode: selectClassNameMode,
          classResolver: selectClassResolver,
        })}
        {...props}
      />
      <ChevronDownIcon
        className={resolveStyledNativeSelectClassName({
          className: iconClassName,
          defaultClassName: nativeSelectClassNames.slot2,
          classNameMode: iconClassNameMode,
          classResolver: iconClassResolver,
        })}
        aria-hidden="true"
        data-slot="native-select-icon"
      />
    </div>
  )
}

function NativeSelectOption({
  mode = DEFAULT_MODE,
  ...props
}: NativeSelectOptionProps) {
  if (mode === "primitive") {
    return <option {...props} />
  }

  return <option data-slot="native-select-option" {...props} />
}

function NativeSelectOptGroup({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: NativeSelectOptGroupProps) {
  if (mode === "primitive") {
    return <optgroup className={className} {...props} />
  }

  return (
    <optgroup
      data-slot="native-select-optgroup"
      className={resolveStyledNativeSelectClassName({
        className,
        defaultClassName: nativeSelectClassNames.slot3,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export { NativeSelect, NativeSelectOptGroup, NativeSelectOption }
