use axum::{Router, routing::get};

use crate::handlers::admin::learning::{
    get_learning_student_detail, get_learning_student_metrics, list_learning_students,
};

pub fn learning_routes() -> Router {
    Router::new()
        .route("/students/metrics", get(get_learning_student_metrics))
        .route("/students", get(list_learning_students))
        .route("/students/{user_id}", get(get_learning_student_detail))
}
