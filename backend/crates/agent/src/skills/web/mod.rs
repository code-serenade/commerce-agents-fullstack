use crate::skills::SkillCategory;

mod conversion;
mod helpers;
mod native_macro;
mod normalize;
mod outcomes;
mod specs;
mod title_scorer;

pub use conversion::WebPageConversion;
pub use native_macro::{
    WebKeywordResearch, WebProductDescription, WebSeoOptimize, WebStoreDesign, WebTitleGenerator,
};
pub use specs::specs;
pub use title_scorer::WebTitleSeoScorer;

fn category() -> SkillCategory {
    SkillCategory::Custom("web".to_owned())
}
