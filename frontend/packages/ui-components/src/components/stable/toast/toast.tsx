import {
  Toaster as CoreToaster,
  toast,
} from "@workspace/ui-core/components/sonner"

export interface ToastProviderProps {
  position?:
    | "top-left"
    | "top-center"
    | "top-right"
    | "bottom-left"
    | "bottom-center"
    | "bottom-right"
}

export function ToastProvider({
  position = "top-center",
}: ToastProviderProps) {
  return <CoreToaster position={position} richColors />
}

export { toast }
