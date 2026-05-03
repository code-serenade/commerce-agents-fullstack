use db_core::PaginatedResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
    pub total_pages: u64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T> From<PaginatedResponse<T>> for PageResponse<T> {
    fn from(response: PaginatedResponse<T>) -> Self {
        Self {
            items: response.items,
            page: response.page,
            page_size: response.page_size,
            total: response.total,
            total_pages: response.total_pages,
            has_next: response.has_next,
            has_prev: response.has_prev,
        }
    }
}
