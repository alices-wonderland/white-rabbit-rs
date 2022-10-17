use std::sync::Arc;

use crate::task::{Input, ServiceTask, Task};
use backend_shared::models::{
  account, journal, record, record_item, user, Account, IntoPresentation, Journal, Record, User,
};
use backend_shared::services::{
  AbstractReadService, AuthUser, ContainingUserQuery, FindAllInput, FindPageInput, JournalQuery, JournalService, Order,
  RecordCommand, RecordCommandCreate, RecordCommandUpdate, RecordService, Sort, FIELD_ADMINS,
};
use fake::faker::chrono::en::Date;
use fake::Fake;
use itertools::Itertools;
use lazy_static::lazy_static;
use rust_decimal::Decimal;
use sea_orm_migration::sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

use std::collections::HashSet;
use std::str::FromStr;

lazy_static! {
  pub static ref TASKS: Vec<ServiceTask<RecordService>> = vec![
    Task::FindById(Input {
      name: "Find Record By Id".to_owned(),
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
        let journals = JournalService::find_all(&*conn, &auth_user, Default::default()).await?;
        let record = journals[0].find_related(Record).one(&*conn).await?.unwrap();
        Ok(record.id)
      }))),
      checker: Arc::new(Box::new(|(conn, _, input, output)| Box::pin(async move {
        let record = Record::find().filter(record::Column::Id.eq(input)).one(&*conn).await?;
        assert_eq!(record, output?);
        Ok(())
      }))),
    }),
    Task::FindPage(Input {
      name: "Find Record Page".to_owned(),
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
            field: "date".to_owned(),
            order: Order::Asc,
          },
        })
      }))),
      checker: Arc::new(Box::new(|(conn, auth_user, _, output)| Box::pin(async move {
        for (a, b) in output?.items.iter().tuple_windows() {
          let record = record::Model::from(a.item.clone());
          let journal = record.find_related(Journal).one(&*conn).await?.unwrap();

          assert!(JournalService::contain_user(&*conn, &journal, auth_user.id().parse::<uuid::Uuid>()?, None).await?);
          assert!(a.item.date < b.item.date);
        }
        Ok(())
      }))),
    }),
    Task::Handle(Input {
      name: "Create Record".to_owned(),
      auth_user: Arc::new(Box::new(|conn| Box::pin(async move {
        let (_, user) = Journal::find()
          .find_also_linked(journal::JournalUserAdmin)
          .one(&*conn)
          .await?
          .unwrap();
        Ok(AuthUser::User(user.unwrap()))
      }))),
      input: Arc::new(Box::new(|(conn, auth_user)| Box::pin(async move {
        let journals = JournalService::find_all(
          &*conn,
          &auth_user,
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
        let accounts: Vec<_> = journals[0]
          .find_related(Account)
          .filter(account::Column::IsArchived.eq(false))
          .all(&*conn)
          .await?;

        Ok(RecordCommand::Create(RecordCommandCreate {
          target_id: None,
          journal_id: journals[0].id,
          name: "new record name".to_owned(),
          description: "new record description".to_owned(),
          typ: record::Type::Record,
          date: Date().fake(),
          tags: HashSet::from_iter(vec!["tag 1".to_owned(), "tag 2".to_owned()]),
          items: HashSet::from_iter(vec![
            record_item::Presentation {
              account_id: accounts[0].id,
              amount: Decimal::from_str("10").unwrap(),
              price: Some(Decimal::from_str("20").unwrap()),
            },
            record_item::Presentation {
              account_id: accounts[1].id,
              amount: Decimal::from_str("100").unwrap(),
              price: Some(Decimal::from_str("2").unwrap()),
            },
          ]),
        }))
      }))),
      checker: Arc::new(Box::new(|(conn, auth_user, input, output)| Box::pin(async move {
        match (&*auth_user, input) {
          (AuthUser::User(_), RecordCommand::Create(input)) => {
            let record = output?.unwrap().into_presentation(&*conn).await?;
            assert_eq!(input.journal_id, record.journal_id);
            assert_eq!(input.name, record.name);
            assert_eq!(input.description, record.description);
            assert_eq!(input.typ, record.typ);
            assert_eq!(input.date, record.date);
            assert_eq!(input.tags, record.tags);
            assert_eq!(input.items, record.items);
            Ok(())
          }
          _ => unreachable!(),
        }
      }))),
    }),
    Task::HandleAll(Input {
      name: "Create, Update and Delete".to_owned(),
      auth_user: Arc::new(Box::new(|conn| Box::pin(async move {
        let (_, user) = Journal::find()
          .find_also_linked(journal::JournalUserAdmin)
          .one(&*conn)
          .await?
          .unwrap();
        Ok(AuthUser::User(user.unwrap()))
      }))),
      input: Arc::new(Box::new(|(conn, auth_user)| Box::pin(async move {
        let journals = JournalService::find_all(
          &*conn,
          &auth_user,
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
        let accounts: Vec<_> = journals[0]
          .find_related(Account)
          .filter(account::Column::IsArchived.eq(false))
          .all(&*conn)
          .await?;

        let lid = uuid::Uuid::new_v4();
        Ok(vec![
          RecordCommand::Create(RecordCommandCreate {
            target_id: Some(lid),
            journal_id: journals[0].id,
            name: "new record name".to_owned(),
            description: "new record description".to_owned(),
            typ: record::Type::Record,
            date: Date().fake(),
            tags: HashSet::from_iter(vec!["tag 1".to_owned(), "tag 2".to_owned()]),
            items: HashSet::from_iter(vec![
              record_item::Presentation {
                account_id: accounts[0].id,
                amount: Decimal::from_str("10").unwrap(),
                price: Some(Decimal::from_str("20").unwrap()),
              },
              record_item::Presentation {
                account_id: accounts[1].id,
                amount: Decimal::from_str("100").unwrap(),
                price: Some(Decimal::from_str("2").unwrap()),
              },
            ]),
          }),
          RecordCommand::Update(RecordCommandUpdate {
            target_id: lid,
            name: None,
            description: Some("new new description".to_owned()),
            typ: None,
            date: Some(Date().fake()),
            tags: Some(HashSet::from_iter(vec!["new tag".to_owned()])),
            items: Some(HashSet::from_iter(vec![
              record_item::Presentation {
                account_id: accounts[2].id,
                amount: Decimal::from_str("5").unwrap(),
                price: Some(Decimal::from_str("10").unwrap()),
              },
              record_item::Presentation {
                account_id: accounts[3].id,
                amount: Decimal::from_str("25").unwrap(),
                price: Some(Decimal::from_str("4").unwrap()),
              },
            ])),
          }),
          RecordCommand::Delete(lid),
        ])
      }))),
      checker: Arc::new(Box::new(|(_, _, input, output)| Box::pin(async move {
        match (&*input, &*output?) {
          (
            &[RecordCommand::Create(ref create), RecordCommand::Update(ref update), RecordCommand::Delete(_)],
            &[Some(ref create_result), Some(ref update_result), None],
          ) => {
            assert_eq!(create.name, create_result.name);
            assert_eq!(create.description, create_result.description);
            assert_eq!(create.typ, create_result.typ);
            assert_eq!(create.date, create_result.date);
            assert_eq!(create.tags, create_result.tags);
            assert_eq!(create.items, create_result.items);

            assert_eq!(create.name, update_result.name);
            assert_eq!(update.description.as_ref().unwrap(), &update_result.description);
            assert_eq!(create.typ, update_result.typ);
            assert_eq!(update.date.as_ref().unwrap(), &update_result.date);
            assert_eq!(update.tags.as_ref().unwrap(), &update_result.tags);
            assert_eq!(update.items.as_ref().unwrap(), &update_result.items);

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
  use backend_shared::services::RecordService;

  #[tokio::test]
  async fn test_tasks() -> backend_shared::Result<()> {
    crate::tests::run_test::<RecordService>(&TASKS).await
  }
}
