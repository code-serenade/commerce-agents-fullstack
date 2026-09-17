import {
  DataTable,
  MetricCards,
  type MetricCardsItem,
} from "@workspace/app-kit"
import type {
  LearningQuestionMetricRow,
  LearningQuestionTableQuery,
} from "../../types"
import type { useLearningQuestionTable } from "../index"

export function LearningQuestionTableView({
  metricCards,
  table,
}: {
  metricCards: readonly MetricCardsItem[]
  table: ReturnType<typeof useLearningQuestionTable>
}) {
  return (
    <div className="min-w-0 space-y-3">
      <MetricCards
        className="grid-cols-2 lg:grid-cols-3 2xl:grid-cols-6"
        items={metricCards}
      />
      <div className="flex h-[calc(100vh-32rem)] min-h-128 min-w-0 flex-1 overflow-hidden">
        <DataTable<LearningQuestionMetricRow, LearningQuestionTableQuery>
          bulkDelete={table.bulkDelete}
          bulkUpdate={table.bulkUpdate}
          columns={table.columns}
          compactColumns={table.compactColumns}
          compactRows={table.compactRows}
          fetchData={table.fetchData}
          fillWidth={table.fillWidth}
          getRowId={table.getRowId}
          height="100%"
          initialPageSize={table.initialPageSize}
          initialQuery={table.initialQuery}
          initialSort={table.initialSort}
          pageSizeOptions={table.pageSizeOptions}
          query={table.query}
          stickyColumns={{ left: 1 }}
        />
      </div>
    </div>
  )
}
