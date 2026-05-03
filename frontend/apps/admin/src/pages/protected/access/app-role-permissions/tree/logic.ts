import type { TreeNode } from "@workspace/ui-components"
import type { RolePermissionTreeNode } from "@/api"
import type { TFunction } from "i18next"
import {
  getPermissionLabel,
  getPermissionSearchText,
} from "../../permission-label"
import type { PermissionSummaryType } from "../types"

export function toTreeNode(node: RolePermissionTreeNode, t: TFunction): TreeNode {
  return {
    id: String(node.id),
    label: getPermissionLabel(t, node),
    searchText: getPermissionSearchText(t, node),
    children: node.children.map((child) => toTreeNode(child, t)),
  }
}

export function flattenPermissionTree(
  nodes: readonly RolePermissionTreeNode[]
): RolePermissionTreeNode[] {
  return nodes.flatMap((node) => [
    node,
    ...flattenPermissionTree(node.children),
  ])
}

export function collectCheckedPermissionIds(
  nodes: readonly RolePermissionTreeNode[]
): string[] {
  return flattenPermissionTree(nodes)
    .filter((node) => node.checked)
    .map((node) => String(node.id))
}

export function buildPermissionSummary(
  selectedResources: readonly RolePermissionTreeNode[]
): Record<PermissionSummaryType, number> {
  const counts: Record<PermissionSummaryType, number> = {
    group: 0,
    action: 0,
  }

  for (const resource of selectedResources) {
    counts[resource.kind] += 1
  }

  return counts
}
