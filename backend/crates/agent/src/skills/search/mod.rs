mod config;
mod output;
mod provider;
mod queries;
mod skills;
mod specs;

pub use config::SearchAdapterConfig;
pub use skills::{
    SearchCompetitor, SearchIndustryBenchmarks, SearchMarketInfo, SearchPlatformPolicy,
    SearchProductReviews, SearchTrends,
};
pub use specs::specs;
