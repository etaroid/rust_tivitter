use rust_tivitter::models::{
    create_pool, create_tokio_runtime, setup_tables, DB_STRING_PRODUCTION,
};

fn main() -> anyhow::Result<()> {
    let tokio_rt = create_tokio_runtime();
    tokio_rt.block_on(run())
}

async fn run() -> anyhow::Result<()> {
    // 本番DBにsession管理用のテーブルを作成
    let session_store = async_sqlx_session::MySqlSessionStore::new(DB_STRING_PRODUCTION).await?;
    session_store.migrate().await?;

    // 本番DB接続するコネクションプールを作成
    let pool = create_pool(DB_STRING_PRODUCTION).await?;

    // tablesを作成
    setup_tables(&pool).await;
    Ok(())
}
