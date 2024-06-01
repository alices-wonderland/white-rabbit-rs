use crate::interceptors::check_auth;
use crate::journal::pb::Journal;
use crate::map_err;
use backend_core::entity::{account, journal, ReadRoot};
use pb::account_service_server::{AccountService, AccountServiceServer};
use pb::{
  Account, AccountQuery, AccountType, FindAllRequest, FindAllResponse, FindByIdRequest,
  FindByIdResponse,
};
use sea_orm::DatabaseConnection;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tonic::codec::CompressionEncoding;
use tonic::codegen::InterceptedService;
use tonic::transport::server::Routes;
use tonic::{Code, Request, Response, Status};
use tonic_reflection::server::Builder;
use uuid::Uuid;

pub(crate) mod pb {
  tonic::include_proto!("whiterabbit.account.v1");

  pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("account_descriptor");
}

impl prost::Name for Account {
  const PACKAGE: &'static str = "whiterabbit.account.v1";
  const NAME: &'static str = "Account";
}

impl From<Vec<account::Root>> for FindAllResponse {
  fn from(results: Vec<account::Root>) -> Self {
    Self {
      values: results.into_iter().map(|model| model.into()).collect(),
      included: Default::default(),
    }
  }
}

impl From<account::Root> for Account {
  fn from(value: account::Root) -> Self {
    Self {
      id: value.id.to_string(),
      journal_id: value.journal_id.to_string(),
      created_date: Some(
        (SystemTime::now() + Duration::from_secs(rand::random::<u64>() % 1_000_000)).into(),
      ),
      name: value.name,
      description: value.description,
      unit: value.unit,
      r#type: AccountType::from(value.typ).into(),
      tags: Vec::from_iter(value.tags),
    }
  }
}

impl From<account::Root> for FindByIdResponse {
  fn from(value: account::Root) -> Self {
    Self { value: Some(value.into()), included: Default::default() }
  }
}

impl From<AccountType> for Option<account::Type> {
  fn from(value: AccountType) -> Self {
    match value {
      AccountType::Income => Some(account::Type::Income),
      AccountType::Expense => Some(account::Type::Expense),
      AccountType::Asset => Some(account::Type::Asset),
      AccountType::Liability => Some(account::Type::Liability),
      AccountType::Equity => Some(account::Type::Equity),
      AccountType::Unspecified => None,
    }
  }
}

impl From<account::Type> for AccountType {
  fn from(value: account::Type) -> Self {
    match value {
      account::Type::Income => AccountType::Income,
      account::Type::Expense => AccountType::Expense,
      account::Type::Asset => AccountType::Asset,
      account::Type::Liability => AccountType::Liability,
      account::Type::Equity => AccountType::Equity,
    }
  }
}

impl TryFrom<AccountQuery> for account::Query {
  type Error = Status;

  fn try_from(value: AccountQuery) -> Result<Self, Self::Error> {
    Ok(Self {
      id: value
        .id
        .iter()
        .map(|v| v.parse())
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|_| Status::new(Code::Internal, "Invalid UUID"))?,
      journal_id: value
        .journal_id
        .iter()
        .map(|v| v.parse())
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|_| Status::new(Code::Internal, "Invalid UUID"))?,

      name: HashSet::from_iter(value.name.clone()),
      unit: value.unit.clone(),
      typ: AccountType::try_from(value.r#type).ok().and_then(|v| v.into()),
      tags: value.tags.iter().cloned().collect(),
      full_text: value.full_text.clone(),
    })
  }
}

#[derive(Debug)]
pub struct AccountServiceImpl {
  pub db: Arc<DatabaseConnection>,
}

#[tonic::async_trait]
impl AccountService for AccountServiceImpl {
  async fn find_all(
    &self,
    request: Request<FindAllRequest>,
  ) -> Result<Response<FindAllResponse>, Status> {
    let query = request.get_ref().query.clone().unwrap_or_default();
    let results = account::Root::find_all(self.db.as_ref(), Some(query.try_into()?), None, None)
      .await
      .map_err(map_err)?;

    let journal_ids = results.iter().map(|model| model.journal_id).collect::<HashSet<_>>();
    let journals = journal::Root::find_all(
      self.db.as_ref(),
      Some(journal::Query { id: journal_ids, ..Default::default() }),
      None,
      None,
    )
    .await
    .map_err(map_err)?;

    Ok(Response::new(FindAllResponse {
      included: journals
        .into_iter()
        .map(Journal::from)
        .filter_map(|model| {
          if let Ok(packed) = prost_types::Any::from_msg(&model) {
            Some((model.id.to_string(), packed))
          } else {
            None
          }
        })
        .collect::<HashMap<_, _>>(),
      values: results.into_iter().map(Account::from).collect::<Vec<_>>(),
    }))
  }

  async fn find_by_id(
    &self,
    request: Request<FindByIdRequest>,
  ) -> Result<Response<FindByIdResponse>, Status> {
    let id: Uuid =
      request.get_ref().id.parse().map_err(|_| Status::new(Code::Internal, "Invalid UUID"))?;

    let model = account::Root::find_one(
      self.db.as_ref(),
      Some(account::Query { id: HashSet::from_iter([id]), ..Default::default() }),
    )
    .await
    .map_err(map_err)?;

    let related_journal = if let Some(model) = &model {
      journal::Root::find_one(
        self.db.as_ref(),
        Some(journal::Query { id: HashSet::from_iter([model.journal_id]), ..Default::default() }),
      )
      .await
      .map_err(map_err)?
    } else {
      None
    };

    Ok(Response::new(FindByIdResponse {
      included: related_journal
        .into_iter()
        .map(Journal::from)
        .filter_map(|model| {
          if let Ok(packed) = prost_types::Any::from_msg(&model) {
            Some((model.id.to_string(), packed))
          } else {
            None
          }
        })
        .collect::<HashMap<_, _>>(),
      value: model.map(Account::from),
    }))
  }
}

pub(crate) fn init(
  reflection_builder: Builder,
  routes: Routes,
  db: Arc<DatabaseConnection>,
) -> (Builder, Routes) {
  let service = InterceptedService::new(
    AccountServiceServer::new(AccountServiceImpl { db })
      .send_compressed(CompressionEncoding::Gzip)
      .accept_compressed(CompressionEncoding::Gzip),
    check_auth,
  );

  (
    reflection_builder.register_encoded_file_descriptor_set(pb::FILE_DESCRIPTOR_SET),
    routes.add_service(service),
  )
}
