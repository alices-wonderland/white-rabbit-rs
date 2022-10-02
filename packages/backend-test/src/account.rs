use std::sync::Arc;

use crate::task::{Input, ServiceTask, Task};
use backend_shared::models::{account, journal, user, Account, IntoPresentation, Journal, User};
use backend_shared::services::{
  AbstractReadService, AccountCommand, AccountCommandCreate, AccountCommandUpdate, AccountService, AuthUser,
  ContainingUserQuery, FindAllInput, FindPageInput, JournalQuery, JournalService, Order, Sort, FIELD_ADMINS,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use sea_orm_migration::sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use std::collections::HashSet;

lazy_static! {
  pub static ref TASKS: Vec<ServiceTask<AccountService>> = vec![
    Task::FindById(Input {
      name: "Find Account By Id".to_owned(),
      auth_user: Arc::new(Box::new(|conn| Box::pin(async move {
        let journal = Journal::find()
          .filter(journal::Column::IsArchived.eq(false))
          .one(&*conn)
          .await?
          .unwrap();
        let user = journal
          .find_linked(journal::JournalUserMember)
          .one(&*conn)
          .await?
          .unwrap();
        Ok(AuthUser::User(user))
      }))),
      input: Arc::new(Box::new(|(conn, auth_user)| Box::pin(async move {
        let journals = JournalService::find_all(&*conn, &*auth_user, Default::default()).await?;
        let account = journals[0].find_related(Account).one(&*conn).await?.unwrap();
        Ok(account.id)
      }))),
      checker: Arc::new(Box::new(|(conn, _, input, output)| Box::pin(async move {
        let account = Account::find()
          .filter(account::Column::Id.eq(input))
          .one(&*conn)
          .await?;
        assert_eq!(account, output?);
        Ok(())
      }))),
    }),
    Task::FindPage(Input {
      name: "Find Account Page".to_owned(),
      auth_user: Arc::new(Box::new(|conn| Box::pin(async move {
        let user = User::find()
          .filter(user::Column::Role.eq(user::Role::User))
          .one(&*conn)
          .await?
          .unwrap();
        Ok(AuthUser::User(user))
      }))),
      input: Arc::new(Box::new(|_| Box::pin(async move {
        Ok(FindPageInput {
          query: None,
          pagination: None,
          sort: Sort {
            field: "name".to_owned(),
            order: Order::Asc,
          },
        })
      }))),
      checker: Arc::new(Box::new(|(conn, auth_user, _, output)| Box::pin(async move {
        for (a, b) in output?.items.iter().tuple_windows() {
          let account = account::Model::from(a.item.clone());
          let journal = account.find_related(Journal).one(&*conn).await?.unwrap();

          assert!(JournalService::contain_user(&*conn, &journal, auth_user.id().parse::<uuid::Uuid>()?, None).await?);
          assert!(a.item.name < b.item.name);
        }
        Ok(())
      }))),
    }),
    Task::Handle(Input {
      name: "Create Account".to_owned(),
      auth_user: Arc::new(Box::new(|conn| Box::pin(async move {
        let user = User::find()
          .filter(user::Column::Role.eq(user::Role::User))
          .one(&*conn)
          .await?
          .unwrap();
        Ok(AuthUser::User(user))
      }))),
      input: Arc::new(Box::new(|(conn, auth_user)| Box::pin(async move {
        let journals = JournalService::find_all(
          &*conn,
          &*auth_user,
          FindAllInput {
            query: Some(JournalQuery {
              containing_user: Some(ContainingUserQuery::Object {
                id: auth_user.id().parse::<uuid::Uuid>()?,
                fields: Some(vec![FIELD_ADMINS.to_owned()]),
              }),
              ..Default::default()
            }),
            ..Default::default()
          },
        )
        .await?;

        Ok(AccountCommand::Create(AccountCommandCreate {
          target_id: None,
          journal_id: journals[0].id,
          name: "new journal name".to_owned(),
          description: "new journal description".to_owned(),
          typ: account::Type::Asset,
          strategy: account::Strategy::Fifo,
          unit: "UNIT".to_owned(),
          tags: HashSet::from_iter(vec!["tag 1".to_owned(), "tag 2".to_owned()]),
        }))
      }))),
      checker: Arc::new(Box::new(|(conn, auth_user, input, output)| Box::pin(async move {
        match (&*auth_user, input) {
          (AuthUser::User(_), AccountCommand::Create(input)) => {
            let account = output?.unwrap().into_presentation(&*conn).await?;
            assert_eq!(input.journal_id, account.journal_id);
            assert_eq!(input.name, account.name);
            assert_eq!(input.description, account.description);
            assert_eq!(input.typ, account.typ);
            assert_eq!(input.strategy, account.strategy);
            assert_eq!(input.unit, account.unit);
            assert_eq!(input.tags, account.tags);
            Ok(())
          }
          _ => unreachable!(),
        }
      }))),
    }),
    Task::HandleAll(Input {
      name: "Create, Update and Delete".to_owned(),
      auth_user: Arc::new(Box::new(|conn| Box::pin(async move {
        let user = User::find()
          .filter(user::Column::Role.eq(user::Role::User))
          .one(&*conn)
          .await?
          .unwrap();
        Ok(AuthUser::User(user))
      }))),
      input: Arc::new(Box::new(|(conn, auth_user)| Box::pin(async move {
        let journals = JournalService::find_all(
          &*conn,
          &*auth_user,
          FindAllInput {
            query: Some(JournalQuery {
              containing_user: Some(ContainingUserQuery::Object {
                id: auth_user.id().parse::<uuid::Uuid>()?,
                fields: Some(vec![FIELD_ADMINS.to_owned()]),
              }),
              ..Default::default()
            }),
            ..Default::default()
          },
        )
        .await?;

        let lid = uuid::Uuid::new_v4();
        Ok(vec![
          AccountCommand::Create(AccountCommandCreate {
            target_id: Some(lid),
            journal_id: journals[0].id,
            name: "new journal name".to_owned(),
            description: "new journal description".to_owned(),
            typ: account::Type::Asset,
            strategy: account::Strategy::Fifo,
            unit: "UNIT".to_owned(),
            tags: HashSet::from_iter(vec!["tag 1".to_owned(), "tag 2".to_owned()]),
          }),
          AccountCommand::Update(AccountCommandUpdate {
            target_id: lid,
            name: Some("new new name".to_owned()),
            description: Some("new new description".to_owned()),
            typ: Some(account::Type::Equity),
            strategy: Some(account::Strategy::Average),
            unit: Some("NEW".to_owned()),
            tags: Some(HashSet::from_iter(vec!["new tag 1".to_owned(), "tag 2".to_owned()])),
            is_archived: Some(true),
          }),
          AccountCommand::Delete(lid),
        ])
      }))),
      checker: Arc::new(Box::new(|(_, _, input, output)| Box::pin(async move {
        match (&*input, &*output?) {
          (
            &[AccountCommand::Create(ref create), AccountCommand::Update(ref update), AccountCommand::Delete(_)],
            &[Some(ref create_result), Some(ref update_result), None],
          ) => {
            assert_eq!(create.name, create_result.name);
            assert_eq!(create.description, create_result.description);
            assert_eq!(create.typ, create_result.typ);
            assert_eq!(create.strategy, create_result.strategy);
            assert_eq!(create.unit, create_result.unit);
            assert_eq!(create.tags, create_result.tags);
            assert!(!create_result.is_archived);

            assert_eq!(update.name.as_ref().unwrap(), &update_result.name);
            assert_eq!(update.description.as_ref().unwrap(), &update_result.description);
            assert_eq!(update.typ.as_ref().unwrap(), &update_result.typ);
            assert_eq!(update.strategy.as_ref().unwrap(), &update_result.strategy);
            assert_eq!(update.unit.as_ref().unwrap(), &update_result.unit);
            assert_eq!(update.tags.as_ref().unwrap(), &update_result.tags);
            assert_eq!(update.is_archived.unwrap(), update_result.is_archived);

            Ok(())
          }
          _ => unreachable!(),
        }
      }))),
    })
  ];
}

#[cfg(test)]
mod tests {
  use super::TASKS;
  use backend_shared::services::AccountService;

  #[tokio::test]
  async fn test_tasks() -> anyhow::Result<()> {
    crate::tests::run_test::<AccountService>(&TASKS).await
  }
}
