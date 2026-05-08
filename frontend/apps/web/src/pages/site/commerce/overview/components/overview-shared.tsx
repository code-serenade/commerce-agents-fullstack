import { Link } from "react-router"
import { Button } from "@workspace/ui-components"

export function LinkButton({ label, to }: { label: string; to: string }) {
  return (
    <Link to={to}>
      <Button size="sm" variant="outline">
        {label}
      </Button>
    </Link>
  )
}
