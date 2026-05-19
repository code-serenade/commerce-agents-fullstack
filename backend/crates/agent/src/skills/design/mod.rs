mod material;
mod native;
mod output;
mod specs;

pub use material::DesignMaterialSpec;
pub use native::{DesignCampaignPoster, DesignColorScheme, DesignDetailPage, DesignMainImage};
pub use specs::{
    design_campaign_poster_spec, design_color_scheme_spec, design_detail_page_spec,
    design_main_image_spec, design_material_spec_spec, specs,
};
