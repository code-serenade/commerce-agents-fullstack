interface TableGuideProps {
  description: string
  title: string
}

export function TableGuide({ description, title }: TableGuideProps) {
  return (
    <div className="mt-4 rounded-(--ui-radius-md) border border-(--app-border) bg-(--app-surface) px-4 py-3">
      <p className="text-sm font-medium">{title}</p>
      <p className="mt-1 text-sm text-(--app-muted-text)">{description}</p>
    </div>
  )
}
