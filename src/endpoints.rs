use crate::models::User;
use async_session::{Session, SessionStore};
use async_sqlx_session::MySqlSessionStore;
use axum::{
    extract::{Extension, FromRequest, Json, RequestParts},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use sqlx::mysql::MySqlQueryResult;
use sqlx::{Error, MySql, Pool};
use std::future::Future;
use std::sync::Arc;

pub async fn run_server(
    arc_pool: Arc<Pool<MySql>>,
    session_store: MySqlSessionStore,
) -> anyhow::Result<()> {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8888));
    let app = Router::new()
        .route("/api/users", post(create_user))
        .route("/api/sessions", post(create_session))
        .layer(Extension(arc_pool))
        .layer(Extension(session_store));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

pub struct CurrentSession(Session);
const AXUM_SESSION_COOKIE_KEY: &'static str = "axum_session";

#[derive(serde::Deserialize)]
pub struct CreateUserParams {
    pub name: String,
}

pub(crate) async fn create_user(
    Json(payload): Json<CreateUserParams>,
    arc_pool: Extension<Arc<Pool<MySql>>>,
) -> impl IntoResponse {
    let user = User {
        id: None,
        name: payload.name,
    };

    match user.insert(&arc_pool).await {
        Ok(_res) => StatusCode::CREATED,
        Err(_e) => StatusCode::BAD_REQUEST,
    }
}

#[derive(serde::Deserialize)]
pub struct CreateSessionParams {
    pub name: String,
}

pub(crate) async fn create_session(
    Json(payload): Json<CreateSessionParams>,
    arc_pool: Extension<Arc<Pool<MySql>>>,
    session_store: Extension<MySqlSessionStore>,
    mut cookie_jar: CookieJar,
) -> impl IntoResponse {
    match User::find_by_name(&payload.name, &arc_pool).await {
        // TODO: nestを浅くする
        Ok(user) => {
            match user {
                Some(user) => {
                    let mut session = Session::new();
                    let expire_seconds = 86400;
                    session.expire_in(std::time::Duration::from_secs(expire_seconds));
                    session.insert("user_id", user.id).unwrap();
                    match session_store.store_session(session).await {
                        Ok(cookie_value) => Ok((
                            StatusCode::CREATED,
                            cookie_jar.add(
                                Cookie::build(AXUM_SESSION_COOKIE_KEY, cookie_value.unwrap())
                                    .secure(false) // TODO: ssl化
                                    .http_only(true)
                                    .same_site(cookie::SameSite::Lax)
                                    .max_age(cookie::time::Duration::new(expire_seconds as i64, 0))
                                    .finish(),
                            ),
                        )),
                        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
                    }
                }
                None => Err(StatusCode::BAD_REQUEST),
            }
        }
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}
