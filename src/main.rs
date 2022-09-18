use rust_tivitter::endpoints::run_server;
use rust_tivitter::models::{create_pool, create_tokio_runtime, DB_STRING_PRODUCTION};
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    let tokio_rt = create_tokio_runtime();
    tokio_rt.block_on(run())
}

async fn run() -> anyhow::Result<()> {
    // DBコネクションプールをnew
    let arc_pool = Arc::new(create_pool(DB_STRING_PRODUCTION).await?);

    // DBにSessionストアをnew
    let session_store = async_sqlx_session::MySqlSessionStore::new(DB_STRING_PRODUCTION).await?;

    // API Serverの起動
    run_server(arc_pool, session_store).await
}
