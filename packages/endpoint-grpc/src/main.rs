use backend_core::entity::{journal, ReadRoot, FIELD_ID};
use backend_core::error::ProblemDetailDef;
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
use uuid::Uuid;

pub mod pb {
  tonic::include_proto!("whiterabbit.journal");

  pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("journal_descriptor");
}

fn map_err(value: backend_core::Error) -> Status {
  let value: ProblemDetailDef = value.into();
  let code = match value.status {
    401 => Code::Unauthenticated,
    404 => Code::NotFound,
    _ => Code::Unknown,
  };
  let details = serde_json::to_string(&value).unwrap_or_default();
  Status::with_details(code, value.detail, details.into())
}

impl From<Vec<journal::Root>> for JournalsResponse {
  fn from(results: Vec<journal::Root>) -> Self {
    Self { values: results.into_iter().map(|model| model.into()).collect() }
  }
}

impl From<journal::Root> for Journal {
  fn from(model: journal::Root) -> Self {
    Self {
      id: model.id.to_string(),
      created_date: None,
      name: model.name,
      description: model.description,
      unit: model.unit,
      tags: Vec::from_iter(model.tags),
    }
  }
}

impl TryFrom<JournalQuery> for journal::Query {
  type Error = Status;

  fn try_from(value: JournalQuery) -> Result<Self, Self::Error> {
    Ok(Self {
      id: value
        .id
        .iter()
        .map(|v| v.parse())
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|_| Status::new(Code::Internal, "Invalid UUID"))?,
      name: HashSet::from_iter(value.name.clone()),
      unit: value.unit.clone(),
      tags: value.tags.iter().cloned().collect(),
      full_text: value.full_text.clone(),
    })
  }
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
    let results =
      journal::Root::find_all(self.db.as_ref(), Some(query.clone().try_into()?), None, None)
        .await
        .map_err(map_err)?;

    Ok(Response::new(results.into()))
  }

  async fn find_by_id(&self, request: Request<String>) -> Result<Response<Journal>, Status> {
    let id: Uuid =
      request.get_ref().parse().map_err(|_| Status::new(Code::Internal, "Invalid UUID"))?;
    if let Some(model) = journal::Root::find_one(
      self.db.as_ref(),
      Some(journal::Query { id: HashSet::from_iter([id]), ..Default::default() }),
    )
    .await
    .map_err(map_err)?
    .map(|model| model.into())
    {
      Ok(Response::new(model))
    } else {
      Err(map_err(backend_core::Error::NotFound(backend_core::error::ErrorNotFound {
        entity: journal::TYPE.to_string(),
        values: vec![(FIELD_ID.to_string(), id.to_string())],
      })))
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
