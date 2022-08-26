use backend_shared::run;
use std::env;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
  env_logger::init();
  log::info!("Run DB in endpoint_graphql: {}", env::var("WHITE_RABBIT_DATABASE_URL")?);
  let _db = run().await?;
  Ok(())
}
