mod account;
mod interceptors;
mod journal;

use backend_core::error::ProblemDetailDef;
use backend_core::init;
use std::sync::Arc;
use tonic::transport::server::Routes;
use tonic::{transport::Server, Code, Status};
use tonic_reflection::server::Builder;

pub(crate) fn map_err(value: backend_core::Error) -> Status {
  let value: ProblemDetailDef = value.into();
  let code = match value.status {
    401 => Code::Unauthenticated,
    404 => Code::NotFound,
    _ => Code::Unknown,
  };
  let details = serde_json::to_string(&value).unwrap_or_default();
  Status::with_details(code, value.detail, details.into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let db = Arc::new(init(".desktop.test.env").await?);
  let addr = "[::1]:50051".parse()?;
  let reflection = Builder::configure();

  let routes = Routes::default();
  let (reflection, routes) = crate::journal::init(reflection, routes, db.clone());
  let (reflection, routes) = crate::account::init(reflection, routes, db);

  let reflection = reflection.build().unwrap();
  Server::builder().add_routes(routes).add_service(reflection).serve(addr).await?;

  Ok(())
}
