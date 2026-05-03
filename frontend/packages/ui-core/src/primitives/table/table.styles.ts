export const tableClassNames = {
  slot1: "relative w-full overflow-x-auto",
  slot2: "w-full caption-bottom text-sm",
  slot3: "[&_tr]:border-b",
  slot4: "[&_tr:last-child]:border-0",
  slot5: "border-t bg-muted/50 font-medium [&>tr]:last:border-b-0",
  slot6:
    "border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted",
  slot7:
    "h-10 px-2 text-left align-middle font-medium whitespace-nowrap text-foreground [&:has([role=checkbox])]:pr-0",
  slot8: "p-2 align-middle whitespace-nowrap [&:has([role=checkbox])]:pr-0",
  slot9: "mt-4 text-sm text-muted-foreground",
} as const
