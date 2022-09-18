use crate::models::{User, UserTweet};
use async_session::{Session, SessionStore as _};
use async_sqlx_session::MySqlSessionStore;
use axum::{
    extract::{Extension, FromRequest, Json, RequestParts},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use sqlx::{MySql, Pool};
use std::sync::Arc;

/////////////////////////////////////////////////////////////////////////////
// API Server Setting & Run
/////////////////////////////////////////////////////////////////////////////
pub async fn run_server(
    arc_pool: Arc<Pool<MySql>>,
    session_store: MySqlSessionStore,
) -> anyhow::Result<()> {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8888));
    let app = Router::new()
        .route("/api/users", post(create_user))
        .route("/api/sessions", post(create_session))
        .route("/api/user_tweets", post(create_user_tweet))
        .layer(Extension(arc_pool))
        .layer(Extension(session_store));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

pub struct CurrentSession(Session);
const AXUM_SESSION_COOKIE_KEY: &'static str = "axum_session";
// https://github.com/tokio-rs/axum/blob/main/examples/sessions/src/main.rsを改変
// axumのカスタムextractorを定義
// クッキーに格納されたセッションキーからセッションデータを復元する
#[axum::async_trait]
impl<B> FromRequest<B> for CurrentSession
where
    B: Send,
{
    type Rejection = StatusCode;
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // MySQLセッションストアを参照する
        let Extension(store) = Extension::<MySqlSessionStore>::from_request(req)
            .await
            .unwrap();
        // ブラウザから送信されたクッキーを参照する
        let cookie = CookieJar::from_request(req).await.unwrap();
        // クッキーからセッションキーを取得
        let session_id = cookie
            .get(AXUM_SESSION_COOKIE_KEY)
            .map(|cookie| cookie.value())
            .unwrap_or("")
            .to_string();
        // セッションキーからセッションデータを復元する
        let session_data = store.load_session(session_id).await;
        match session_data {
            Ok(session_data) => match session_data {
                // セッションデータが存在＝セッションデータを返す
                Some(session_data) => Ok(CurrentSession(session_data)),
                // セッションデータが存在しない＝ログインできていない
                None => Err(StatusCode::UNAUTHORIZED),
            },
            // RDBとの接続が切れている可能性がある、500を返す
            Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
// User API
/////////////////////////////////////////////////////////////////////////////
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

/////////////////////////////////////////////////////////////////////////////
// Session API
/////////////////////////////////////////////////////////////////////////////
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

/////////////////////////////////////////////////////////////////////////////
// User Tweet API
/////////////////////////////////////////////////////////////////////////////
#[derive(serde::Deserialize)]
pub struct CreateUserTweetParams {
    pub content: String,
}

pub(crate) async fn create_user_tweet(
    Json(payload): Json<CreateUserTweetParams>,
    arc_pool: Extension<Arc<Pool<MySql>>>,
    session: CurrentSession,
) -> impl IntoResponse {
    match session.0.get::<u64>("user_id") {
        None => Err(StatusCode::UNAUTHORIZED),
        Some(user_id) => {
            let tweet = UserTweet {
                id: None,
                content: payload.content,
                user_id,
            };
            match tweet.insert(&arc_pool).await {
                Ok(_) => Ok(StatusCode::CREATED),
                Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
            }
        }
    }
}
