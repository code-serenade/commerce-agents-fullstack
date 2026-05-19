mod calendar;
mod helpers;
mod native_macro;
mod normalize;
mod outcomes;
mod specs;
mod title_scorer;

pub use calendar::CreativeContentCalendar;
pub use native_macro::{
    CreativeIpBranding, CreativeLiveScript, CreativeProductArticle, CreativeSeedingCopy,
    CreativeTrendCatch, CreativeVideoScript,
};
pub use specs::specs;
pub use title_scorer::CreativeTitleCtrScorer;

use crate::skills::SkillCategory;

pub(super) fn category() -> SkillCategory {
    SkillCategory::Custom("creative".to_owned())
}
