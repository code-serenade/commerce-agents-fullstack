export const itemClassNames = {
  slot1:
    "group/item flex w-full flex-wrap items-center rounded-lg border text-sm transition-colors duration-100 outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 [a]:transition-colors [a]:hover:bg-muted",
  slot2:
    "flex shrink-0 items-center justify-center gap-2 group-has-data-[slot=item-description]/item:translate-y-0.5 group-has-data-[slot=item-description]/item:self-start [&_svg]:pointer-events-none",
  slot3: "border-transparent",
  slot4: "border-border",
  slot5: "border-transparent bg-muted/50",
  slot6: "gap-2.5 px-3 py-2.5",
  slot7: "gap-2 px-2.5 py-2 in-data-[slot=dropdown-menu-content]:p-0",
  slot8: "bg-transparent",
  slot9: "[&_svg:not([class*='size-'])]:size-4",
  slot10:
    "size-10 overflow-hidden rounded-sm group-data-[size=sm]/item:size-8 group-data-[size=xs]/item:size-6 [&_img]:size-full [&_img]:object-cover",
} as const
