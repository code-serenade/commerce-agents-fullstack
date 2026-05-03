import { useState, type ReactNode } from "react"
import { Camera } from "lucide-react"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@workspace/ui-core/components/alert-dialog"
import {
  Avatar,
  AvatarFallback,
  AvatarImage,
} from "@workspace/ui-core/components/avatar"
import { Button } from "@workspace/ui-core/components/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@workspace/ui-core/components/dropdown-menu"
import { AvatarUploadField, type AvatarUploadResult } from "./avatar-upload-field"

export interface AvatarPhotoEditorLabels {
  edit: ReactNode
  upload: ReactNode
  uploading: ReactNode
  remove: ReactNode
  removeConfirmTitle: ReactNode
  removeConfirmDescription: ReactNode
  removeConfirmCancel: ReactNode
  removeConfirmAction: ReactNode
}

export interface AvatarPhotoEditorProps {
  value?: string
  alt: string
  fallback: ReactNode
  disabled?: boolean
  labels: AvatarPhotoEditorLabels
  onUploaded: (result: AvatarUploadResult) => void | Promise<void>
  onRemove: () => void | Promise<void>
}

export function AvatarPhotoEditor({
  value,
  alt,
  fallback,
  disabled = false,
  labels,
  onUploaded,
  onRemove,
}: AvatarPhotoEditorProps) {
  const [menuOpen, setMenuOpen] = useState(false)
  const [removeConfirmOpen, setRemoveConfirmOpen] = useState(false)

  return (
    <div className="relative w-fit">
      <Avatar className="size-48 bg-[#0f1724] ring-1 ring-(--app-border) sm:size-56">
        {value ? <AvatarImage src={value} alt={alt} /> : null}
        <AvatarFallback className="bg-[#1c2431] text-6xl font-semibold text-white">
          {fallback}
        </AvatarFallback>
      </Avatar>

      <AvatarUploadField
        value={value}
        alt={alt}
        fallback={fallback}
        disabled={disabled}
        onUploaded={async (result) => {
          await onUploaded(result)
          setMenuOpen(false)
        }}
        renderTrigger={({ openFileDialog, disabled, uploading }) => (
          <DropdownMenu open={menuOpen} onOpenChange={setMenuOpen}>
            <DropdownMenuTrigger asChild>
              <Button
                type="button"
                variant="default"
                size="lg"
                disabled={disabled}
                className="absolute right-3 bottom-4 shadow-(--ui-shadow-soft)"
              >
                <Camera className="size-4" />
                {labels.edit}
              </Button>
            </DropdownMenuTrigger>

            <DropdownMenuContent align="end" className="w-44">
              <DropdownMenuItem
                disabled={disabled}
                onSelect={(event) => {
                  event.preventDefault()
                  setMenuOpen(false)
                  window.setTimeout(openFileDialog, 0)
                }}
              >
                {uploading ? labels.uploading : labels.upload}
              </DropdownMenuItem>
              <DropdownMenuItem
                disabled={disabled || !value}
                onSelect={() => {
                  setMenuOpen(false)
                  setRemoveConfirmOpen(true)
                }}
              >
                {labels.remove}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      />

      <AlertDialog
        open={removeConfirmOpen}
        onOpenChange={setRemoveConfirmOpen}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{labels.removeConfirmTitle}</AlertDialogTitle>
            <AlertDialogDescription>
              {labels.removeConfirmDescription}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{labels.removeConfirmCancel}</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => void onRemove()}
            >
              {labels.removeConfirmAction}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
