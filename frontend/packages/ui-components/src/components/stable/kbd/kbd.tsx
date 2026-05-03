import { Kbd as CoreKbd, KbdGroup as CoreKbdGroup } from "@workspace/ui-core/components/kbd"

export interface KbdProps {
  keys: string[]
}

export function Kbd({ keys }: KbdProps) {
  return (
    <CoreKbdGroup>
      {keys.map((key, index) => (
        <CoreKbd key={`${key}-${index}`}>{key}</CoreKbd>
      ))}
    </CoreKbdGroup>
  )
}
