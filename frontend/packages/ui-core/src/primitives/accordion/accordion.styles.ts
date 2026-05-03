export const accordionClassNames = {
  slot1: "flex",
  slot2:
    "pointer-events-none shrink-0 group-aria-expanded/accordion-trigger:hidden",
  slot3:
    "pointer-events-none hidden shrink-0 group-aria-expanded/accordion-trigger:inline",
  slot4:
    "overflow-hidden text-sm data-open:animate-accordion-down data-closed:animate-accordion-up",
  slot5: "flex w-full flex-col",
  slot6: "not-last:border-b",
  slot7:
    "group/accordion-trigger relative flex flex-1 items-start justify-between rounded-lg border border-transparent py-2.5 text-left text-sm font-medium transition-all outline-none hover:underline focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:after:border-ring disabled:pointer-events-none disabled:opacity-50 **:data-[slot=accordion-trigger-icon]:ml-auto **:data-[slot=accordion-trigger-icon]:size-4 **:data-[slot=accordion-trigger-icon]:text-muted-foreground",
  slot8:
    "h-(--radix-accordion-content-height) pt-0 pb-2.5 [&_a]:underline [&_a]:underline-offset-3 [&_a]:hover:text-foreground [&_p:not(:last-child)]:mb-4",
} as const
