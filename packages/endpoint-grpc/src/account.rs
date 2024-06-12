use crate::journal::pb::Journal;
use crate::{decode_strings, decode_uuid, decode_uuids, map_err};
use backend_core::entity::{account, journal, ReadRoot};
use itertools::Itertools;
use pb::account_service_server::{AccountService, AccountServiceServer};
use pb::{
  account_command, Account, AccountCommand, AccountCommandBatch, AccountCommandCreate,
  AccountCommandDelete, AccountCommandUpdate, AccountQuery, AccountType, FindAllRequest,
  FindAllResponse, FindByIdRequest, FindByIdResponse, HandleCommandRequest, HandleCommandResponse,
};
use sea_orm::DatabaseConnection;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tonic::codec::CompressionEncoding;
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

// Model -> Proto

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

fn decode_type(value: i32) -> Result<account::Type, Status> {
  if let Ok(value) = AccountType::try_from(value) {
    match value {
      AccountType::Income => return Ok(account::Type::Income),
      AccountType::Expense => return Ok(account::Type::Expense),
      AccountType::Asset => return Ok(account::Type::Asset),
      AccountType::Liability => return Ok(account::Type::Liability),
      AccountType::Equity => return Ok(account::Type::Equity),
      _ => {}
    }
  }

  Err(Status::invalid_argument("Invalid Account type"))
}

// Proto -> Model

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
      id: decode_uuids(value.id)?,
      journal_id: decode_uuids(value.journal_id)?,
      name: HashSet::from_iter(value.name),
      unit: value.unit,
      typ: decode_type(value.r#type).ok(),
      tags: value.tags.iter().cloned().collect(),
      full_text: value.full_text,
    })
  }
}

impl TryFrom<AccountCommandCreate> for account::CommandCreate {
  type Error = Status;

  fn try_from(value: AccountCommandCreate) -> Result<Self, Self::Error> {
    Ok(account::CommandCreate {
      journal_id: decode_uuid(value.journal_id)?,
      name: value.name,
      description: value.description,
      unit: value.unit,
      typ: decode_type(value.r#type)?,
      tags: HashSet::from_iter(value.tags),
    })
  }
}

impl TryFrom<AccountCommandUpdate> for account::CommandUpdate {
  type Error = Status;

  fn try_from(value: AccountCommandUpdate) -> Result<Self, Self::Error> {
    Ok(account::CommandUpdate {
      id: decode_uuid(value.id)?,
      name: value.name,
      description: value.description,
      unit: value.unit,
      typ: decode_type(value.r#type).ok(),
      tags: value.tags.map(decode_strings),
    })
  }
}

impl TryFrom<AccountCommandDelete> for account::CommandDelete {
  type Error = Status;

  fn try_from(value: AccountCommandDelete) -> Result<Self, Self::Error> {
    Ok(account::CommandDelete { id: decode_uuids(value.id)? })
  }
}

impl TryFrom<AccountCommandBatch> for account::CommandBatch {
  type Error = Status;

  fn try_from(value: AccountCommandBatch) -> Result<Self, Self::Error> {
    Ok(account::CommandBatch {
      create: value.create.into_iter().map(|c| c.try_into()).try_collect()?,
      update: value.update.into_iter().map(|c| c.try_into()).try_collect()?,
      delete: decode_uuids(value.delete)?,
    })
  }
}

impl TryFrom<account_command::Command> for account::Command {
  type Error = Status;

  fn try_from(value: account_command::Command) -> Result<Self, Self::Error> {
    Ok(match value {
      account_command::Command::Create(command) => account::Command::Create(command.try_into()?),
      account_command::Command::Update(command) => account::Command::Update(command.try_into()?),
      account_command::Command::Delete(command) => account::Command::Delete(command.try_into()?),
      account_command::Command::Batch(command) => account::Command::Batch(command.try_into()?),
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

  async fn handle_command(
    &self,
    request: Request<HandleCommandRequest>,
  ) -> Result<Response<HandleCommandResponse>, Status> {
    if let Some(AccountCommand { command: Some(command) }) = request.get_ref().command.clone() {
      let command: account::Command = command.try_into()?;
      let values = account::Root::handle(self.db.as_ref(), command)
        .await
        .map_err(map_err)?
        .into_iter()
        .map(|model| model.into())
        .collect();
      Ok(Response::new(HandleCommandResponse { values }))
    } else {
      Err(Status::invalid_argument("Command field should not be empty"))
    }
  }
}

pub(crate) fn init(
  reflection_builder: Builder,
  routes: Routes,
  db: Arc<DatabaseConnection>,
) -> (Builder, Routes) {
  let service = AccountServiceServer::new(AccountServiceImpl { db })
    .send_compressed(CompressionEncoding::Gzip)
    .accept_compressed(CompressionEncoding::Gzip);

  (
    reflection_builder.register_encoded_file_descriptor_set(pb::FILE_DESCRIPTOR_SET),
    routes.add_service(service),
  )
}
