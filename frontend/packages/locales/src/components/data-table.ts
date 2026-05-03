import type { SupportedLanguage } from "../index"
import { localeResources } from "../resources"

export interface DataTableCopy {
  emptyText: string
  errorText: string
  loadingText: string
  refreshLabel: string
  resetLabel: string
  totalLabel: string
  insertLabel: string
  actionsLabel: string
  editLabel: string
  deleteLabel: string
  moreLabel: string
  cancelLabel: string
  saveLabel: string
  confirmDeleteLabel: string
  deleteDialogTitle: string
  deleteDialogDescription: string
  bulkDeleteDialogDescription: (count: number) => string
  sortAscendingLabel: string
  sortDescendingLabel: string
  clearSortLabel: string
  bulkDeleteLabel: (count: number) => string
  bulkUpdateLabel: (count: number) => string
  bulkUpdateTitle: string
  bulkUpdateDescription: (count: number) => string
  bulkUpdateFieldLabel: string
  bulkUpdateValueLabel: string
  bulkUpdateCancelLabel: string
  bulkUpdateApplyLabel: string
  createdAtLabel: string
  updatedAtLabel: string
  auditQueryLabel: string
  auditFieldPlaceholder: string
  auditRangePlaceholder: string
  createdAtRangePlaceholder: string
  updatedAtRangePlaceholder: string
  auditEmptyText: string
}

export function getDataTableCopy(language: SupportedLanguage) {
  const copy = localeResources[language].components.dataTable

  return {
    ...copy,
    bulkDeleteDialogDescription: (count: number) =>
      language === "zhCN"
        ? `该操作会删除已选的 ${count} 条数据，且无法撤销。`
        : `This action will delete ${count} selected row(s) and cannot be undone.`,
    bulkDeleteLabel: (count: number) =>
      language === "zhCN" ? `删除已选 (${count})` : `Delete Selected (${count})`,
    bulkUpdateLabel: (count: number) =>
      language === "zhCN" ? `批量修改 (${count})` : `Bulk Update (${count})`,
    bulkUpdateDescription: (count: number) =>
      language === "zhCN"
        ? `对 ${count} 条已选数据应用相同的值。`
        : `Apply the same value to ${count} selected row(s).`,
  }
}
