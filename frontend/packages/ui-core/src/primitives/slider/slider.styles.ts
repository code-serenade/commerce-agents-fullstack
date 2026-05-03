export const sliderClassNames = {
  slot0:
    "relative flex w-full touch-none items-center select-none py-3 data-disabled:opacity-50 data-vertical:h-full data-vertical:min-h-40 data-vertical:w-auto data-vertical:flex-col",
  slot1:
    "relative grow overflow-hidden rounded-full bg-zinc-500/60 data-horizontal:h-1.5 data-horizontal:w-full data-vertical:h-full data-vertical:w-1",
  slot2:
    "absolute bg-[#2f7eff] select-none data-horizontal:h-full data-vertical:w-full",
  slot3:
    "relative block h-[16px] w-[20px] shrink-0 rounded-[999px] border border-white/70 bg-zinc-100 shadow-[0_1px_4px_rgba(0,0,0,0.22)] transition-[color,box-shadow,transform] select-none after:absolute after:inset-0 after:rounded-[999px] hover:shadow-[0_0_0_1px_rgba(47,126,255,0.32),0_1px_4px_rgba(0,0,0,0.22)] focus-visible:shadow-[0_0_0_1px_rgba(47,126,255,0.5),0_1px_4px_rgba(0,0,0,0.22)] focus-visible:outline-hidden active:scale-95 disabled:pointer-events-none disabled:opacity-50",
  slot4:
    "pointer-events-none absolute inset-x-0 top-[calc(50%+8px)] h-[3px] bg-[radial-gradient(circle,_rgba(148,163,184,0.65)_1px,_transparent_1.1px)] bg-[length:36px_3px] bg-repeat-x",
} as const
