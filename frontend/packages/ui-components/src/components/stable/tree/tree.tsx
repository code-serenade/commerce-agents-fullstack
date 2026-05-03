import {
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type ReactNode,
  type SVGProps,
} from "react"
import { Checkbox as CoreCheckbox } from "@workspace/ui-core/components/checkbox"
import { Input as CoreInput } from "@workspace/ui-core/components/input"
import {
  Popover as CorePopover,
  PopoverContent as CorePopoverContent,
  PopoverTrigger as CorePopoverTrigger,
} from "@workspace/ui-core/components/popover"
import { ScrollArea as CoreScrollArea } from "@workspace/ui-core/components/scroll-area"
import { cn } from "../../../lib/utils"

export interface TreeNode {
  id: string
  label: ReactNode
  children?: TreeNode[]
  disabled?: boolean
  searchText?: string
}

export interface TreeViewProps {
  data: TreeNode[]
  value: string[]
  onValueChange: (value: string[]) => void
  expandedIds?: string[]
  defaultExpandedIds?: string[]
  onExpandedIdsChange?: (value: string[]) => void
  disabled?: boolean
  cascade?: boolean
  searchValue?: string
  emptyLabel?: ReactNode
  maxHeight?: number
}

export interface TreeProps extends Omit<TreeViewProps, "searchValue"> {
  placeholder?: string
  searchPlaceholder?: string
  emptySearchLabel?: ReactNode
  open?: boolean
  defaultOpen?: boolean
  onOpenChange?: (open: boolean) => void
}

interface TreeIndexEntry {
  node: TreeNode
  parentId: string | null
}

type CheckedState = boolean | "indeterminate"

function ChevronDownIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" {...props}>
      <path d="m6 9 6 6 6-6" />
    </svg>
  )
}

function ChevronRightIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" {...props}>
      <path d="m9 6 6 6-6 6" />
    </svg>
  )
}

function SearchIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" {...props}>
      <circle cx="11" cy="11" r="7" />
      <path d="m20 20-3.5-3.5" />
    </svg>
  )
}

function XIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" {...props}>
      <path d="M18 6 6 18" />
      <path d="m6 6 12 12" />
    </svg>
  )
}

function getNodeText(node: TreeNode) {
  if (typeof node.searchText === "string") {
    return node.searchText.toLowerCase()
  }

  if (typeof node.label === "string") {
    return node.label.toLowerCase()
  }

  return ""
}

function buildTreeIndex(
  nodes: TreeNode[],
  parentId: string | null = null,
  index: Map<string, TreeIndexEntry> = new Map(),
) {
  for (const node of nodes) {
    index.set(node.id, { node, parentId })

    if (node.children?.length) {
      buildTreeIndex(node.children, node.id, index)
    }
  }

  return index
}

function collectDescendantIds(node: TreeNode): string[] {
  const ids = [node.id]

  for (const child of node.children ?? []) {
    ids.push(...collectDescendantIds(child))
  }

  return ids
}

function areAllChildrenChecked(node: TreeNode, selectedIds: Set<string>) {
  const children = node.children ?? []

  if (!children.length) {
    return selectedIds.has(node.id)
  }

  return children.every((child) => selectedIds.has(child.id))
}

function syncAncestors(nodeId: string, index: Map<string, TreeIndexEntry>, selectedIds: Set<string>) {
  let currentParentId = index.get(nodeId)?.parentId ?? null

  while (currentParentId) {
    const parentEntry = index.get(currentParentId)

    if (!parentEntry) {
      break
    }

    if (areAllChildrenChecked(parentEntry.node, selectedIds)) {
      selectedIds.add(currentParentId)
    } else {
      selectedIds.delete(currentParentId)
    }

    currentParentId = parentEntry.parentId
  }
}

function toggleNodeSelection(params: {
  node: TreeNode
  checked: boolean
  cascade: boolean
  selectedIds: Set<string>
  index: Map<string, TreeIndexEntry>
}) {
  const {
    node,
    checked,
    cascade,
    selectedIds,
    index,
  } = params

  if (cascade) {
    const ids = collectDescendantIds(node)

    for (const id of ids) {
      if (checked) {
        selectedIds.add(id)
      } else {
        selectedIds.delete(id)
      }
    }
  } else if (checked) {
    selectedIds.add(node.id)
  } else {
    selectedIds.delete(node.id)
  }

  syncAncestors(node.id, index, selectedIds)

  return Array.from(selectedIds)
}

function getNodeCheckState(node: TreeNode, selectedIds: Set<string>): CheckedState {
  if (selectedIds.has(node.id)) {
    return true
  }

  const children = node.children ?? []

  if (!children.length) {
    return false
  }

  const hasCheckedChild = children.some((child) => {
    const state = getNodeCheckState(child, selectedIds)
    return state === true || state === "indeterminate"
  })

  return hasCheckedChild ? "indeterminate" : false
}

function filterNodes(nodes: TreeNode[], query: string): TreeNode[] {
  const normalizedQuery = query.trim().toLowerCase()

  if (!normalizedQuery) {
    return nodes
  }

  return nodes.flatMap((node) => {
    const filteredChildren = filterNodes(node.children ?? [], normalizedQuery)
    const matched = getNodeText(node).includes(normalizedQuery)

    if (!matched && filteredChildren.length === 0) {
      return []
    }

    return [
      {
        ...node,
        children: filteredChildren,
      },
    ]
  })
}

function collectTopSelectedNodes(nodes: TreeNode[], selectedIds: Set<string>) {
  const result: TreeNode[] = []

  const visit = (list: TreeNode[], hasSelectedAncestor: boolean) => {
    for (const node of list) {
      const selected = selectedIds.has(node.id)
      const nextHasSelectedAncestor = hasSelectedAncestor || selected

      if (selected && !hasSelectedAncestor) {
        result.push(node)
      }

      if (node.children?.length) {
        visit(node.children, nextHasSelectedAncestor)
      }
    }
  }

  visit(nodes, false)

  return result
}

function useControllableState<T>({
  value,
  defaultValue,
  onChange,
}: {
  value: T | undefined
  defaultValue: T
  onChange?: (value: T) => void
}) {
  const [internalValue, setInternalValue] = useState(defaultValue)
  const isControlled = value !== undefined
  const resolvedValue = isControlled ? value : internalValue

  const setValue = (nextValue: T) => {
    if (!isControlled) {
      setInternalValue(nextValue)
    }

    onChange?.(nextValue)
  }

  return [resolvedValue, setValue] as const
}

interface TreeNodeRowProps {
  node: TreeNode
  depth: number
  disabled: boolean
  forcedExpanded: boolean
  expandedIds: Set<string>
  selectedIds: Set<string>
  onToggleExpand: (nodeId: string) => void
  onToggleSelect: (node: TreeNode, checked: boolean) => void
}

function TreeNodeRow({
  node,
  depth,
  disabled,
  forcedExpanded,
  expandedIds,
  selectedIds,
  onToggleExpand,
  onToggleSelect,
}: TreeNodeRowProps) {
  const hasChildren = Boolean(node.children?.length)
  const expanded = forcedExpanded || expandedIds.has(node.id)
  const checkedState = getNodeCheckState(node, selectedIds)
  const rowDisabled = disabled || node.disabled

  return (
    <div className="space-y-0">
      <div
        className={cn(
          "group flex min-h-6.5 items-center gap-1.5 rounded-md px-2 text-sm",
          !rowDisabled && "hover:bg-muted/50",
          rowDisabled && "cursor-not-allowed opacity-60",
        )}
        style={{ paddingInlineStart: 8 + depth * 20 }}
      >
        <button
          type="button"
          aria-label={expanded ? "Collapse node" : "Expand node"}
          className={cn(
            "inline-flex size-5 shrink-0 items-center justify-center rounded text-muted-foreground",
            hasChildren ? "hover:bg-muted" : "invisible",
          )}
          onClick={() => {
            if (hasChildren) {
              onToggleExpand(node.id)
            }
          }}
          disabled={!hasChildren}
        >
          {hasChildren ? (
            expanded ? (
              <ChevronDownIcon className="size-4" />
            ) : (
              <ChevronRightIcon className="size-4" />
            )
          ) : null}
        </button>

        <CoreCheckbox
          checked={checkedState}
          disabled={rowDisabled}
          onCheckedChange={(value) => {
            onToggleSelect(node, value === true || value === "indeterminate")
          }}
        />

        <button
          type="button"
          className="min-w-0 flex-1 truncate py-0 text-left"
          onClick={() => onToggleSelect(node, checkedState !== true)}
          disabled={rowDisabled}
        >
          {node.label}
        </button>
      </div>

      {hasChildren && expanded ? (
        <div className="space-y-0">
          {node.children?.map((child) => (
            <TreeNodeRow
              key={child.id}
              node={child}
              depth={depth + 1}
              disabled={disabled}
              forcedExpanded={forcedExpanded}
              expandedIds={expandedIds}
              selectedIds={selectedIds}
              onToggleExpand={onToggleExpand}
              onToggleSelect={onToggleSelect}
            />
          ))}
        </div>
      ) : null}
    </div>
  )
}

export function TreeView({
  data,
  value,
  onValueChange,
  expandedIds,
  defaultExpandedIds = [],
  onExpandedIdsChange,
  disabled = false,
  cascade = true,
  searchValue = "",
  emptyLabel = "No nodes found.",
  maxHeight = 320,
}: TreeViewProps) {
  const index = useMemo(() => buildTreeIndex(data), [data])
  const [resolvedExpandedIds, setResolvedExpandedIds] = useControllableState({
    value: expandedIds,
    defaultValue: defaultExpandedIds,
    onChange: onExpandedIdsChange,
  })

  const filteredData = useMemo(() => filterNodes(data, searchValue), [data, searchValue])
  const selectedIds = useMemo(() => new Set(value), [value])
  const expandedIdSet = useMemo(() => new Set(resolvedExpandedIds), [resolvedExpandedIds])
  const forcedExpanded = searchValue.trim().length > 0

  const handleToggleExpand = (nodeId: string) => {
    const nextValue = new Set(resolvedExpandedIds)

    if (nextValue.has(nodeId)) {
      nextValue.delete(nodeId)
    } else {
      nextValue.add(nodeId)
    }

    setResolvedExpandedIds(Array.from(nextValue))
  }

  const handleToggleSelect = (node: TreeNode, checked: boolean) => {
    const nextValue = toggleNodeSelection({
      node,
      checked,
      cascade,
      selectedIds: new Set(value),
      index,
    })

    onValueChange(nextValue)
  }

  if (!filteredData.length) {
    return (
      <div className="flex items-center justify-center rounded-lg border border-dashed px-4 py-8 text-sm text-muted-foreground">
        {emptyLabel}
      </div>
    )
  }

  return (
    <CoreScrollArea className="w-full">
      <div style={{ maxHeight }} className="space-y-0 pr-3">
        {filteredData.map((node) => (
          <TreeNodeRow
            key={node.id}
            node={node}
            depth={0}
            disabled={disabled}
            forcedExpanded={forcedExpanded}
            expandedIds={expandedIdSet}
            selectedIds={selectedIds}
            onToggleExpand={handleToggleExpand}
            onToggleSelect={handleToggleSelect}
          />
        ))}
      </div>
    </CoreScrollArea>
  )
}

export function Tree({
  data,
  value,
  onValueChange,
  expandedIds,
  defaultExpandedIds = [],
  onExpandedIdsChange,
  disabled = false,
  cascade = true,
  emptyLabel = "No nodes found.",
  emptySearchLabel = "No matching nodes.",
  maxHeight = 320,
  placeholder = "Select nodes",
  searchPlaceholder = "Search nodes",
  open,
  defaultOpen = false,
  onOpenChange,
}: TreeProps) {
  const triggerId = useId()
  const [searchValue, setSearchValue] = useState("")
  const [resolvedOpen, setResolvedOpen] = useControllableState({
    value: open,
    defaultValue: defaultOpen,
    onChange: onOpenChange,
  })
  const selectedIds = useMemo(() => new Set(value), [value])
  const index = useMemo(() => buildTreeIndex(data), [data])
  const selectedNodes = useMemo(
    () => collectTopSelectedNodes(data, selectedIds),
    [data, selectedIds],
  )
  const triggerRef = useRef<HTMLButtonElement | null>(null)
  const [contentWidth, setContentWidth] = useState<number>()

  useEffect(() => {
    if (!resolvedOpen) {
      return
    }

    setContentWidth(triggerRef.current?.offsetWidth)
  }, [resolvedOpen, value.length])

  return (
    <CorePopover open={resolvedOpen} onOpenChange={setResolvedOpen}>
      <CorePopoverTrigger
        ref={triggerRef}
        id={triggerId}
        type="button"
        disabled={disabled}
        className={cn(
          "flex min-h-11 w-full items-center justify-between gap-3 rounded-xl border bg-background px-3 py-2 text-left text-sm shadow-xs outline-none transition",
          "border-input hover:border-ring/60 focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/30",
          disabled && "cursor-not-allowed opacity-60",
        )}
      >
        <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2">
          {selectedNodes.length ? (
            selectedNodes.map((node) => (
              <span
                key={node.id}
                className="inline-flex max-w-full items-center gap-1 rounded-md bg-muted px-2 py-1 text-sm"
              >
                <span className="truncate">{node.label}</span>
                <span
                  aria-hidden="true"
                  className="inline-flex size-4 items-center justify-center rounded text-muted-foreground hover:text-foreground"
                  onMouseDown={(event) => {
                    event.preventDefault()
                    event.stopPropagation()

                    const nextValue = toggleNodeSelection({
                      node,
                      checked: false,
                      cascade,
                      selectedIds: new Set(value),
                      index,
                    })

                    onValueChange(nextValue)
                  }}
                >
                  <XIcon className="size-3.5" />
                </span>
              </span>
            ))
          ) : (
            <span className="text-muted-foreground">{placeholder}</span>
          )}
        </div>

        <ChevronDownIcon
          className={cn(
            "size-4 shrink-0 text-muted-foreground transition-transform",
            resolvedOpen && "rotate-180",
          )}
        />
      </CorePopoverTrigger>

      <CorePopoverContent
        align="start"
        className="rounded-2xl p-0"
        style={contentWidth ? { width: contentWidth } : undefined}
      >
        <div className="space-y-3 p-3">
          <div className="relative">
            <SearchIcon className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
            <CoreInput
              value={searchValue}
              onChange={(event) => setSearchValue(event.target.value)}
              placeholder={searchPlaceholder}
              className="pl-9"
            />
          </div>

          <TreeView
            data={data}
            value={value}
            onValueChange={onValueChange}
            expandedIds={expandedIds}
            defaultExpandedIds={defaultExpandedIds}
            onExpandedIdsChange={onExpandedIdsChange}
            disabled={disabled}
            cascade={cascade}
            searchValue={searchValue}
            emptyLabel={searchValue.trim() ? emptySearchLabel : emptyLabel}
            maxHeight={maxHeight}
          />
        </div>
      </CorePopoverContent>
    </CorePopover>
  )
}
