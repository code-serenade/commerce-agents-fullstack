import { render, screen, waitFor, within } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"

import { DataTable } from "./data-table"
import type { DataTableColumn, DataTableProps } from "./data-table.types"

interface TestRow {
  id: string
  name: string
}

interface TestQuery {
  keyword: string
}

const rows: TestRow[] = [
  { id: "row-1", name: "Alpha" },
  { id: "row-2", name: "Beta" },
  { id: "row-3", name: "Gamma" },
]

const columns: readonly DataTableColumn<TestRow>[] = [
  {
    key: "name",
    header: "Name",
    renderCell: (row) => row.name,
    width: 180,
  },
]

function renderTable(
  props: Partial<DataTableProps<TestRow, TestQuery>> = {}
) {
  const fetchData = vi.fn().mockResolvedValue({
    items: rows,
    total: rows.length,
  })

  render(
    <DataTable<TestRow, TestQuery>
      columns={columns}
      fetchData={fetchData}
      getRowId={(row) => row.id}
      initialQuery={{ keyword: "" }}
      initialPageSize={10}
      {...props}
    />
  )

  return { fetchData }
}

describe("DataTable", () => {
  it("renders fetched rows", async () => {
    const { fetchData } = renderTable()

    expect(await screen.findByText("Alpha")).toBeInTheDocument()
    expect(screen.getByText("Beta")).toBeInTheDocument()
    expect(screen.getByText("Gamma")).toBeInTheDocument()

    await waitFor(() => {
      expect(fetchData).toHaveBeenCalledTimes(1)
    })
  })

  it("disables striped rows when stripedRows is false", async () => {
    const { container } = render(
      <DataTable<TestRow, TestQuery>
        columns={columns}
        fetchData={async () => ({
          items: rows,
          total: rows.length,
        })}
        getRowId={(row) => row.id}
        initialQuery={{ keyword: "" }}
        initialPageSize={10}
        stripedRows={false}
      />
    )

    await screen.findByText("Alpha")

    const bodyRows = Array.from(container.querySelectorAll("tbody tr"))
    expect(bodyRows.length).toBeGreaterThan(1)
    expect(bodyRows.every((row) => !row.className.includes("bg-muted/40"))).toBe(
      true
    )
  })

  it("applies compact column and row styles", async () => {
    const { container } = render(
      <DataTable<TestRow, TestQuery>
        columns={columns}
        fetchData={async () => ({
          items: rows,
          total: rows.length,
        })}
        getRowId={(row) => row.id}
        initialQuery={{ keyword: "" }}
        initialPageSize={10}
        compactColumns
        compactRows
      />
    )

    await screen.findByText("Alpha")

    const headerCell = container.querySelector("thead th")
    const bodyCell = container.querySelector("tbody td")

    expect(headerCell).toHaveClass("px-1.5")
    expect(headerCell).toHaveClass("h-8")
    expect(bodyCell).toHaveClass("px-1.5")
    expect(bodyCell).toHaveClass("py-1.5")
  })

  it("opens custom dialogs and triggers more menu actions", async () => {
    const user = userEvent.setup()
    const moreAction = vi.fn()

    renderTable({
      insert: {
        label: "新增客户",
        title: "新增客户弹窗",
        renderContent: () => <div>Insert Custom Content</div>,
      },
      rowActions: {
        edit: {
          label: "编辑客户",
          title: (row) => `编辑 ${row.name}`,
          renderContent: ({ row }) => <div>Edit {row.name}</div>,
        },
        moreLabel: "更多操作",
        moreItems: [
          {
            key: "custom-audit",
            label: "打开审计弹窗",
            onClick: (row) => moreAction(row.id),
          },
        ],
      },
    })

    await screen.findByText("Alpha")

    await user.click(screen.getByRole("button", { name: "新增客户" }))
    expect(await screen.findByText("新增客户弹窗")).toBeInTheDocument()
    expect(screen.getByText("Insert Custom Content")).toBeInTheDocument()

    await user.keyboard("{Escape}")
    await waitFor(() => {
      expect(screen.queryByText("新增客户弹窗")).not.toBeInTheDocument()
    })

    const row = screen.getByText("Alpha").closest("tr")
    expect(row).not.toBeNull()

    const rowScope = within(row!)
    await user.click(rowScope.getByRole("button", { name: "编辑客户" }))
    expect(await screen.findByText("编辑 Alpha")).toBeInTheDocument()
    expect(screen.getByText("Edit Alpha")).toBeInTheDocument()

    await user.keyboard("{Escape}")
    await waitFor(() => {
      expect(screen.queryByText("编辑 Alpha")).not.toBeInTheDocument()
    })

    await user.click(rowScope.getByRole("button", { name: "更多操作" }))
    await user.click(
      await screen.findByRole("menuitem", { name: "打开审计弹窗" })
    )

    expect(moreAction).toHaveBeenCalledWith("row-1")
  })
})
