import { Accordion as AccordionPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { ChevronDownIcon, ChevronUpIcon } from "../../lib/icon-slots"
import { accordionClassNames } from "./accordion.styles"
import type {
  AccordionClassResolver,
  AccordionContentProps,
  AccordionItemProps,
  AccordionProps,
  AccordionTriggerProps,
} from "./accordion.types"

function resolveStyledAccordionClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: AccordionClassResolver
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

function Accordion({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: AccordionProps) {
  if (mode === "primitive") {
    return <AccordionPrimitive.Root className={className} {...props} />
  }

  return (
    <AccordionPrimitive.Root
      data-slot="accordion"
      className={resolveStyledAccordionClassName({
        className,
        defaultClassName: accordionClassNames.slot5,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function AccordionItem({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: AccordionItemProps) {
  if (mode === "primitive") {
    return <AccordionPrimitive.Item className={className} {...props} />
  }

  return (
    <AccordionPrimitive.Item
      data-slot="accordion-item"
      className={resolveStyledAccordionClassName({
        className,
        defaultClassName: accordionClassNames.slot6,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function AccordionTrigger({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  headerClassName,
  headerClassNameMode = "merge",
  headerClassResolver,
  collapsedIconClassName,
  collapsedIconClassNameMode = "merge",
  collapsedIconClassResolver,
  expandedIconClassName,
  expandedIconClassNameMode = "merge",
  expandedIconClassResolver,
  children,
  ...props
}: AccordionTriggerProps) {
  if (mode === "primitive") {
    return (
      <AccordionPrimitive.Header className={headerClassName}>
        <AccordionPrimitive.Trigger className={className} {...props}>
          {children}
          <ChevronDownIcon className={collapsedIconClassName} />
          <ChevronUpIcon className={expandedIconClassName} />
        </AccordionPrimitive.Trigger>
      </AccordionPrimitive.Header>
    )
  }

  return (
    <AccordionPrimitive.Header
      className={resolveStyledAccordionClassName({
        className: headerClassName,
        defaultClassName: accordionClassNames.slot1,
        classNameMode: headerClassNameMode,
        classResolver: headerClassResolver,
      })}
    >
      <AccordionPrimitive.Trigger
        data-slot="accordion-trigger"
        className={resolveStyledAccordionClassName({
          className,
          defaultClassName: accordionClassNames.slot7,
          classNameMode,
          classResolver,
        })}
        {...props}
      >
        {children}
        <ChevronDownIcon
          data-slot="accordion-trigger-icon"
          className={resolveStyledAccordionClassName({
            className: collapsedIconClassName,
            defaultClassName: accordionClassNames.slot2,
            classNameMode: collapsedIconClassNameMode,
            classResolver: collapsedIconClassResolver,
          })}
        />
        <ChevronUpIcon
          data-slot="accordion-trigger-icon"
          className={resolveStyledAccordionClassName({
            className: expandedIconClassName,
            defaultClassName: accordionClassNames.slot3,
            classNameMode: expandedIconClassNameMode,
            classResolver: expandedIconClassResolver,
          })}
        />
      </AccordionPrimitive.Trigger>
    </AccordionPrimitive.Header>
  )
}

function AccordionContent({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  innerClassName,
  innerClassNameMode = "merge",
  innerClassResolver,
  children,
  ...props
}: AccordionContentProps) {
  if (mode === "primitive") {
    return (
      <AccordionPrimitive.Content className={className} {...props}>
        <div className={innerClassName}>{children}</div>
      </AccordionPrimitive.Content>
    )
  }

  return (
    <AccordionPrimitive.Content
      data-slot="accordion-content"
      className={resolveStyledAccordionClassName({
        className,
        defaultClassName: accordionClassNames.slot4,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      <div
        className={resolveStyledAccordionClassName({
          className: innerClassName,
          defaultClassName: accordionClassNames.slot8,
          classNameMode: innerClassNameMode,
          classResolver: innerClassResolver,
        })}
      >
        {children}
      </div>
    </AccordionPrimitive.Content>
  )
}

export { Accordion, AccordionItem, AccordionTrigger, AccordionContent }
