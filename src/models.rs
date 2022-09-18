use sqlx::mysql::{MySqlPoolOptions, MySqlQueryResult};
use sqlx::{Executor, MySql, Pool};

// DB接続先情報
pub const DB_STRING_PRODUCTION: &'static str = "mysql://user:password@localhost:53306/production";
pub const DB_STRING_TEST: &'static str = "mysql://user:password@localhost:53306/test";

// 非同期処理を実行するtokio runtimeを作成
pub fn create_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

pub async fn create_pool(url: &str) -> Result<Pool<MySql>, sqlx::Error> {
    MySqlPoolOptions::new().connect(url).await
}

pub async fn setup_tables(pool: &Pool<MySql>) {
    panic_except_duplicate_key(User::create_table(pool).await);
    panic_except_duplicate_key(UserTweet::create_table(pool).await);
    panic_except_duplicate_key(FollowRelation::create_table(pool).await);
}

// MySQLはINDEXにIF NOT EXISTSを宣言できないため、duplicate keyエラー以外の場合にpanicするように自前実装
fn panic_except_duplicate_key(query_result: Result<MySqlQueryResult, sqlx::Error>) {
    if let Err(e) = query_result {
        let is_duplicate_index_error = e
            .as_database_error()
            .unwrap()
            .message()
            .starts_with("Duplicate key name");
        if !is_duplicate_index_error {
            panic!("Error except duplicate key");
        }
    };
}

// User
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Option<u64>,
    pub name: String,
}
impl User {
    pub const TABLE_NAME: &'static str = "users";
    pub async fn create_table(pool: &Pool<MySql>) -> Result<MySqlQueryResult, sqlx::Error> {
        pool.execute(include_str!("../sql/ddl/users_create.sql"))
            .await
    }

    pub async fn find_by_name(name: &str, pool: &Pool<MySql>) -> Result<Option<User>, sqlx::Error> {
        let sql = format!(r#"SELECT * FROM {} WHERE name = ?;"#, Self::TABLE_NAME);
        let result = sqlx::query_as::<_, User>(&sql)
            .bind(name)
            .fetch_optional(pool)
            .await;
        result
    }

    pub async fn insert(&self, pool: &Pool<MySql>) -> Result<MySqlQueryResult, sqlx::Error> {
        let sql = format!(r#"INSERT INTO {} (name) VALUES (?);"#, Self::TABLE_NAME);
        let result = sqlx::query(&sql).bind(&self.name).execute(pool).await;
        result
    }
}

// UserTweet
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct UserTweet {
    pub id: Option<u64>,
    pub user_id: u64,
    pub content: String,
}
impl UserTweet {
    pub const TABLE_NAME: &'static str = "user_tweets";
    pub async fn create_table(pool: &Pool<MySql>) -> Result<MySqlQueryResult, sqlx::Error> {
        pool.execute(include_str!("../sql/ddl/user_tweets_create.sql"))
            .await
    }
}

// FollowRelation
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct FollowRelation {
    pub id: Option<u64>,
    pub followee_id: u64,
    pub follower_id: u64,
}
impl FollowRelation {
    pub const TABLE_NAME: &'static str = "follow_relations";
    pub async fn create_table(pool: &Pool<MySql>) -> Result<MySqlQueryResult, sqlx::Error> {
        pool.execute(include_str!("../sql/ddl/follow_relations_create.sql"))
            .await
    }
}
