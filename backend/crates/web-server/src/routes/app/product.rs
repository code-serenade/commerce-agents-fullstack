use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::app::{
    create_competitor_change, create_product, create_product_competitor, create_product_knowledge,
    create_product_material, delete_product, delete_product_asset, delete_product_competitor,
    delete_product_knowledge, delete_product_material, get_product, get_product_asset_access,
    get_product_competitor, get_product_competitor_analysis, get_product_lifecycle,
    get_product_material, list_competitor_changes, list_product_assets, list_product_competitors,
    list_product_knowledge, list_product_material_versions, list_product_materials, list_products,
    presign_product_asset_upload, product_dashboard, product_lifecycle_options,
    register_product_asset, transition_product_lifecycle, update_product, update_product_asset,
    update_product_competitor, update_product_knowledge, update_product_material,
};

pub fn product_routes() -> Router {
    Router::new()
        .route("/products", get(list_products).post(create_product))
        .route("/products/dashboard", get(product_dashboard))
        .route(
            "/products/lifecycle/options",
            get(product_lifecycle_options),
        )
        .route(
            "/products/{product_id}",
            get(get_product)
                .patch(update_product)
                .delete(delete_product),
        )
        .route(
            "/products/{product_id}/lifecycle",
            get(get_product_lifecycle).patch(transition_product_lifecycle),
        )
        .route(
            "/products/{product_id}/knowledge",
            get(list_product_knowledge).post(create_product_knowledge),
        )
        .route(
            "/products/{product_id}/knowledge/{knowledge_id}",
            axum::routing::patch(update_product_knowledge).delete(delete_product_knowledge),
        )
        .route(
            "/products/{product_id}/assets/presign-upload",
            post(presign_product_asset_upload),
        )
        .route(
            "/products/{product_id}/assets",
            get(list_product_assets).post(register_product_asset),
        )
        .route(
            "/products/{product_id}/assets/{asset_id}",
            axum::routing::patch(update_product_asset).delete(delete_product_asset),
        )
        .route(
            "/products/{product_id}/assets/{asset_id}/access",
            get(get_product_asset_access),
        )
        .route(
            "/products/{product_id}/materials",
            get(list_product_materials).post(create_product_material),
        )
        .route(
            "/products/{product_id}/materials/{material_id}",
            get(get_product_material)
                .patch(update_product_material)
                .delete(delete_product_material),
        )
        .route(
            "/products/{product_id}/materials/{material_id}/versions",
            get(list_product_material_versions),
        )
        .route(
            "/products/{product_id}/competitors",
            get(list_product_competitors).post(create_product_competitor),
        )
        .route(
            "/products/{product_id}/competitors/changes",
            get(list_competitor_changes),
        )
        .route(
            "/products/{product_id}/competitors/analysis",
            get(get_product_competitor_analysis),
        )
        .route(
            "/products/{product_id}/competitors/{competitor_id}",
            get(get_product_competitor)
                .patch(update_product_competitor)
                .delete(delete_product_competitor),
        )
        .route(
            "/products/{product_id}/competitors/{competitor_id}/changes",
            post(create_competitor_change),
        )
}
