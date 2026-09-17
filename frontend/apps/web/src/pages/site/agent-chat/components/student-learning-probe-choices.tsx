import { Check } from "lucide-react"

export function StudentLearningProbeChoices({
  ariaLabelledBy,
  choices,
  disabled,
  selectedChoiceId,
  onSelect,
}: {
  ariaLabelledBy: string
  choices: Array<{ id: string; text: string }>
  disabled: boolean
  selectedChoiceId: string | null
  onSelect: (choiceId: string) => void
}) {
  return (
    <div
      aria-labelledby={ariaLabelledBy}
      className="mb-3 grid gap-2"
      role="radiogroup"
    >
      {choices.map((choice, index) => {
        const selected = selectedChoiceId === choice.id
        return (
          <label
            key={choice.id}
            className={`flex min-h-10 w-full items-start gap-2.5 rounded-md border px-3 py-2 text-left text-xs/5 font-medium transition-colors ${
              selected
                ? "border-primary bg-(--agent-chat-accent-soft) text-foreground"
                : "border-(--agent-chat-soft-border) bg-background text-foreground hover:border-primary/40 hover:bg-(--agent-chat-accent-soft)"
            } ${disabled ? "cursor-wait opacity-60" : "cursor-pointer"}`}
          >
            <input
              type="radio"
              name={ariaLabelledBy}
              value={choice.id}
              checked={selected}
              disabled={disabled}
              onChange={() => onSelect(choice.id)}
              className="peer sr-only"
            />
            <span
              aria-hidden="true"
              className={`mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-full border text-[10px] font-semibold peer-focus-visible:ring-2 peer-focus-visible:ring-ring peer-focus-visible:ring-offset-2 peer-focus-visible:outline-none ${
                selected
                  ? "border-primary bg-primary text-primary-foreground"
                  : "border-border bg-background text-muted-foreground"
              }`}
            >
              {String.fromCharCode(65 + index)}
            </span>
            <span className="min-w-0 flex-1 wrap-break-word">
              {choice.text}
            </span>
            {selected ? (
              <Check
                aria-hidden="true"
                className="mt-1 size-3.5 shrink-0 text-primary"
              />
            ) : null}
          </label>
        )
      })}
    </div>
  )
}
