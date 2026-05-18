// Inference helpers are split by domain while preserving the parent runner module scope.
include!("inference/metrics.rs");
include!("inference/pricing.rs");
include!("inference/promo.rs");
include!("inference/inventory.rs");
include!("inference/campaign.rs");
include!("inference/analytics.rs");
include!("inference/search.rs");
include!("inference/service.rs");
include!("inference/design.rs");
include!("inference/product.rs");
include!("inference/engineering.rs");
include!("inference/task_context.rs");
