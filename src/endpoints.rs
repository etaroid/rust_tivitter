use std::sync::Arc;
use async_sqlx_session::MySqlSessionStore;
use axum::{
    extract::{Extension, FromRequest, Json, RequestParts},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sqlx::{Error, MySql, Pool};
use sqlx::mysql::MySqlQueryResult;
use crate::models::User;

pub async fn run_server(
    arc_pool: Arc<Pool<MySql>>,
    session_store: MySqlSessionStore,
) -> anyhow::Result<()> {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8888));
    let app = Router::new()
        .route("/api/users", post(create_user))
        .layer(Extension(arc_pool))
        .layer(Extension(session_store));

    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct CreateUserParams {
    pub name: String,
}

async fn create_user(
    Json(payload): Json<CreateUserParams>,
    arc_pool: Extension<Arc<Pool<MySql>>>,
) -> impl IntoResponse {
    let user = User {
        id: None,
        name: payload.name
    };

    match user.insert(&arc_pool).await {
        Ok(_res) => StatusCode::CREATED,
        Err(_e) => StatusCode::BAD_REQUEST
    }
}