use crate::interceptors::check_auth;
use crate::map_err;
use backend_core::entity::{journal, ReadRoot};
use pb::journal_service_server::{JournalService, JournalServiceServer};
use pb::{
  FindAllRequest, FindAllResponse, FindByIdRequest, FindByIdResponse, Journal, JournalQuery,
};
use sea_orm::DatabaseConnection;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tonic::codec::CompressionEncoding;
use tonic::codegen::InterceptedService;
use tonic::transport::server::Routes;
use tonic::{Code, Request, Response, Status};
use tonic_reflection::server::Builder;
use uuid::Uuid;

pub(crate) mod pb {
  tonic::include_proto!("whiterabbit.journal.v1");

  pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("journal_descriptor");
}

impl prost::Name for Journal {
  const PACKAGE: &'static str = "whiterabbit.journal.v1";
  const NAME: &'static str = "Journal";
}

impl From<Vec<journal::Root>> for FindAllResponse {
  fn from(results: Vec<journal::Root>) -> Self {
    Self { values: results.into_iter().map(|model| model.into()).collect() }
  }
}

impl From<journal::Root> for Journal {
  fn from(value: journal::Root) -> Self {
    Self {
      id: value.id.to_string(),
      created_date: Some(
        (SystemTime::now() + Duration::from_secs(rand::random::<u64>() % 1_000_000)).into(),
      ),
      name: value.name,
      description: value.description,
      unit: value.unit,
      tags: Vec::from_iter(value.tags),
    }
  }
}

impl From<journal::Root> for FindByIdResponse {
  fn from(value: journal::Root) -> Self {
    Self { value: Some(value.into()) }
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
    request: Request<FindAllRequest>,
  ) -> Result<Response<FindAllResponse>, Status> {
    let query = request.get_ref().query.clone().unwrap_or_default();
    let results = journal::Root::find_all(self.db.as_ref(), Some(query.try_into()?), None, None)
      .await
      .map_err(map_err)?;

    Ok(Response::new(results.into()))
  }

  async fn find_by_id(
    &self,
    request: Request<FindByIdRequest>,
  ) -> Result<Response<FindByIdResponse>, Status> {
    let id: Uuid =
      request.get_ref().id.parse().map_err(|_| Status::new(Code::Internal, "Invalid UUID"))?;

    let model = journal::Root::find_one(
      self.db.as_ref(),
      Some(journal::Query { id: HashSet::from_iter([id]), ..Default::default() }),
    )
    .await
    .map_err(map_err)?
    .map(|model| model.into());

    Ok(Response::new(FindByIdResponse { value: model }))
  }
}

pub(crate) fn init(
  reflection_builder: Builder,
  routes: Routes,
  db: Arc<DatabaseConnection>,
) -> (Builder, Routes) {
  let service = InterceptedService::new(
    JournalServiceServer::new(JournalServiceImpl { db })
      .send_compressed(CompressionEncoding::Gzip)
      .accept_compressed(CompressionEncoding::Gzip),
    check_auth,
  );

  (
    reflection_builder.register_encoded_file_descriptor_set(pb::FILE_DESCRIPTOR_SET),
    routes.add_service(service),
  )
}
