export const inputGroupClassNames = {
  slot1:
    "flex h-auto cursor-text items-center justify-center gap-2 py-1.5 text-sm font-medium text-muted-foreground select-none group-data-[disabled=true]/input-group:opacity-50 [&>kbd]:rounded-[calc(var(--radius)-5px)] [&>svg:not([class*='size-'])]:size-4",
  slot2: "flex items-center gap-2 text-sm shadow-none",
  slot3: "order-first pl-2 has-[>button]:ml-[-0.3rem] has-[>kbd]:ml-[-0.15rem]",
  slot4: "order-last pr-2 has-[>button]:mr-[-0.3rem] has-[>kbd]:mr-[-0.15rem]",
  slot5:
    "order-first w-full justify-start px-2.5 pt-2 group-has-[>input]/input-group:pt-2 [.border-b]:pb-2",
  slot6:
    "order-last w-full justify-start px-2.5 pb-2 group-has-[>input]/input-group:pb-2 [.border-t]:pt-2",
  slot7: "inline-start",
  slot8:
    "h-6 gap-1 rounded-[calc(var(--radius)-3px)] px-1.5 [&>svg:not([class*='size-'])]:size-3.5",
  slot9: "size-6 rounded-[calc(var(--radius)-3px)] p-0 has-[>svg]:p-0",
  slot10: "size-8 p-0 has-[>svg]:p-0",
} as const
