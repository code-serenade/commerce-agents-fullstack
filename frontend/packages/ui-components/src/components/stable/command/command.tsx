import type { ReactNode } from "react"
import {
  Command as CoreCommand,
  CommandEmpty as CoreCommandEmpty,
  CommandGroup as CoreCommandGroup,
  CommandInput as CoreCommandInput,
  CommandItem as CoreCommandItem,
  CommandList as CoreCommandList,
  CommandShortcut as CoreCommandShortcut,
} from "@workspace/ui-core/components/command"

export interface CommandEntry {
  label: ReactNode
  value: string
  shortcut?: ReactNode
  disabled?: boolean
}

export interface CommandSection {
  heading: string
  items: CommandEntry[]
}

export interface CommandProps {
  sections: CommandSection[]
  placeholder?: string
  emptyLabel?: ReactNode
  onSelect?: (value: string) => void
}

export function Command({
  sections,
  placeholder = "Type a command...",
  emptyLabel = "No results found.",
  onSelect,
}: CommandProps) {
  return (
    <CoreCommand>
      <CoreCommandInput placeholder={placeholder} />
      <CoreCommandList>
        <CoreCommandEmpty>{emptyLabel}</CoreCommandEmpty>
        {sections.map((section) => (
          <CoreCommandGroup key={section.heading} heading={section.heading}>
            {section.items.map((item) => (
              <CoreCommandItem
                key={item.value}
                value={item.value}
                disabled={item.disabled}
                onSelect={onSelect}
              >
                {item.label}
                {item.shortcut ? (
                  <CoreCommandShortcut>{item.shortcut}</CoreCommandShortcut>
                ) : null}
              </CoreCommandItem>
            ))}
          </CoreCommandGroup>
        ))}
      </CoreCommandList>
    </CoreCommand>
  )
}
