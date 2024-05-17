use backend_core::entity::{journal, ReadRoot};
use backend_core::init;
use pb::journal_service_server::{JournalService, JournalServiceServer};
use pb::{Journal, JournalQuery, JournalsResponse};
use sea_orm::DatabaseConnection;
use std::collections::HashSet;
use std::sync::Arc;
use tonic::codec::CompressionEncoding;
use tonic::codegen::InterceptedService;
use tonic::metadata::MetadataValue;
use tonic::{transport::Server, Code, Request, Response, Status};
use tonic_reflection::server::Builder;

pub mod pb {
  tonic::include_proto!("whiterabbit.journal");

  pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("journal_descriptor");
}

#[derive(Debug)]
pub struct JournalServiceImpl {
  pub db: Arc<DatabaseConnection>,
}

#[tonic::async_trait]
impl JournalService for JournalServiceImpl {
  async fn find_all(
    &self,
    request: Request<JournalQuery>,
  ) -> Result<Response<JournalsResponse>, Status> {
    let query = request.get_ref();
    log::info!("Journal Query: {:#?}", query);
    let results = journal::Root::find_all(
      self.db.as_ref(),
      Some(journal::Query {
        id: query
          .id
          .iter()
          .map(|v| v.parse())
          .collect::<Result<HashSet<_>, _>>()
          .map_err(|_| Status::new(Code::Internal, "Invalid UUID"))?,
        name: HashSet::from_iter(query.name.clone()),
        unit: query.unit.clone(),
        full_text: query.full_text.clone(),
      }),
      None,
      None,
    )
    .await
    .map_err(|err| Status::new(Code::Internal, err.to_string()))?;

    Ok(Response::new(JournalsResponse {
      values: results
        .into_iter()
        .map(|model| Journal {
          id: model.id.to_string(),
          name: model.name,
          description: model.description,
          unit: model.unit,
          tags: Vec::from_iter(model.tags),
          ..Default::default()
        })
        .collect(),
    }))
  }

  async fn find_by_id(&self, request: Request<String>) -> Result<Response<Journal>, Status> {
    if let Some(model) = journal::Root::find_one(
      self.db.as_ref(),
      Some(journal::Query {
        id: HashSet::from_iter([request
          .get_ref()
          .parse()
          .map_err(|_| Status::new(Code::Internal, "Invalid UUID"))?]),
        ..Default::default()
      }),
    )
    .await
    .map_err(|err| Status::new(Code::Internal, err.to_string()))?
    .map(|model| Journal {
      id: model.id.to_string(),
      name: model.name,
      description: model.description,
      unit: model.unit,
      tags: Vec::from_iter(model.tags),
      ..Default::default()
    }) {
      Ok(Response::new(model))
    } else {
      Err(Status::not_found("Journal Not Found"))
    }
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let db = Arc::new(init(".desktop.test.env").await?);
  let addr = "[::1]:50051".parse()?;
  let reflection = Builder::configure()
    .register_encoded_file_descriptor_set(pb::FILE_DESCRIPTOR_SET)
    .build()
    .unwrap();
  let service = InterceptedService::new(
    JournalServiceServer::new(JournalServiceImpl { db })
      .send_compressed(CompressionEncoding::Gzip)
      .accept_compressed(CompressionEncoding::Gzip),
    check_auth,
  );

  Server::builder().add_service(reflection).add_service(service).serve(addr).await?;

  Ok(())
}

fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
  let token: MetadataValue<_> = "Bearer some-secret-token".parse().unwrap();
  log::info!("Metadata: {:#?}", req.metadata());

  match req.metadata().get("authorization") {
    Some(t) if token == t => Ok(req),
    _ => Err(Status::unauthenticated("No valid auth token")),
  }
}
