use db_core::PaginationParams;

/// 通用分页参数（page + limit）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaginationInput {
    /// 页码（从 1 开始）
    pub page: u64,

    /// 每页数量
    pub limit: u64,
}

impl Default for PaginationInput {
    fn default() -> Self {
        Self { page: 1, limit: 20 }
    }
}

impl PaginationInput {
    /// 转换为 pg-core 的分页参数并进行归一化
    pub fn to_params(self) -> PaginationParams {
        PaginationParams::new(self.page, self.limit).validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range<T> {
    pub from: Option<T>,
    pub to: Option<T>,
}
