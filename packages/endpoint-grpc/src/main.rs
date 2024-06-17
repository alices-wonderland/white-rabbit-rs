mod account;
mod interceptors;
mod journal;

use backend_core::error::ProblemDetailDef;
use backend_core::init;
use itertools::Itertools;
use prost_types::{value, ListValue};
use std::collections::HashSet;
use std::sync::Arc;
use tonic::service::Routes;
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

fn encode_strings(items: impl IntoIterator<Item = impl ToString>) -> ListValue {
  ListValue {
    values: items
      .into_iter()
      .map(|v| prost_types::Value {
        kind: Some(prost_types::value::Kind::StringValue(v.to_string())),
      })
      .collect::<Vec<_>>(),
  }
}

fn decode_strings(value: ListValue) -> HashSet<String> {
  value
    .values
    .into_iter()
    .filter_map(|v| if let Some(value::Kind::StringValue(v)) = v.kind { Some(v) } else { None })
    .collect()
}

fn decode_uuid(value: impl ToString) -> Result<uuid::Uuid, Status> {
  value.to_string().parse().map_err(|_e| Status::new(Code::Internal, "Invalid UUID"))
}

fn decode_uuids(
  values: impl IntoIterator<Item = impl ToString>,
) -> Result<HashSet<uuid::Uuid>, Status> {
  values.into_iter().map(decode_uuid).try_collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let db = Arc::new(init(".env").await?);
  let addr = "[::1]:50051".parse()?;
  let reflection = Builder::configure();

  let routes = Routes::default();
  let (reflection, routes) = crate::journal::init(reflection, routes, db.clone());
  let (reflection, routes) = crate::account::init(reflection, routes, db);

  let reflection = reflection.build().unwrap();
  Server::builder().add_routes(routes).add_service(reflection).serve(addr).await?;

  Ok(())
}
