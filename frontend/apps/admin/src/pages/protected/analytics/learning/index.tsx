import { DataTable, MetricCards } from "@workspace/app-kit"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@workspace/ui-components"
import { useLearningData } from "./learning-data"
import {
  LearningQuestionTableView,
  useLearningQuestionTable,
} from "./question-table"
import { useLearningStudentsTable } from "./table"
import type {
  LearningStudentRow,
  LearningStudentTableQuery,
} from "./types"

type LearningAnalyticsTab = "students" | "questions"

export default function LearningAnalyticsPage() {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = useState<LearningAnalyticsTab>("students")
  const {
    metricCards,
    pilotMetricCards,
    questionGovernanceMetricCards,
    fetchData,
    fetchQuestionData,
  } = useLearningData()
  const table = useLearningStudentsTable(fetchData)
  const questionTable = useLearningQuestionTable(fetchQuestionData)

  return (
    <div className="w-full min-w-0 space-y-4">
      <div className="min-w-0">
        <h1 className="text-2xl font-semibold tracking-normal">
          {t("admin.analytics.learning.heading.title")}
        </h1>
        <p className="mt-1 text-sm text-(--app-muted-text)">
          {t("admin.analytics.learning.heading.description")}
        </p>
      </div>

      <MetricCards
        className="grid-cols-2 lg:grid-cols-3 xl:grid-cols-6"
        items={metricCards}
      />

      <section className="min-w-0 space-y-2">
        <h2 className="text-sm font-semibold tracking-normal">
          {t("admin.analytics.learning.pilotMetrics.title")}
        </h2>
        <MetricCards
          className="grid-cols-2 xl:grid-cols-4"
          items={pilotMetricCards}
        />
      </section>

      <Tabs
        className="min-w-0"
        value={activeTab}
        onValueChange={(value) => setActiveTab(value as LearningAnalyticsTab)}
      >
        <TabsList>
          <TabsTrigger value="students">
            {t("admin.analytics.learning.tabs.students")}
          </TabsTrigger>
          <TabsTrigger value="questions">
            {t("admin.analytics.learning.tabs.questions")}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="students" className="mt-3">
          <div className="flex h-[calc(100vh-25rem)] min-h-160 min-w-0 flex-1 overflow-hidden">
            <DataTable<LearningStudentRow, LearningStudentTableQuery>
              auditColumns={table.auditColumns}
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
              rowActions={table.rowActions}
              stickyColumns={{ left: 1 }}
            />
          </div>
        </TabsContent>

        <TabsContent value="questions" className="mt-3">
          <LearningQuestionTableView
            metricCards={questionGovernanceMetricCards}
            table={questionTable}
          />
        </TabsContent>
      </Tabs>
    </div>
  )
}
