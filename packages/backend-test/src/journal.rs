use std::collections::HashSet;
use std::sync::Arc;

use lazy_static::lazy_static;
use sea_orm_migration::sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

use crate::task::{Input, ServiceTask, Task};
use backend_shared::models::{journal, user, AccessItemType, IntoPresentation, Journal, User};
use backend_shared::services::{
  AbstractReadService, AuthUser, FindAllInput, FindPageInput, JournalCommand, JournalCommandCreate,
  JournalCommandUpdate, JournalService, Order, Sort,
};
use itertools::Itertools;

lazy_static! {
  pub static ref TASKS: Vec<ServiceTask<JournalService>> = vec![
    Task::FindById(Input {
      name: "Find Journal By Id".to_owned(),
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
        Ok(JournalService::find_all(&*conn, &auth_user, FindAllInput::default()).await?[0].id)
      }))),
      checker: Arc::new(Box::new(|(conn, _, input, output)| Box::pin(async move {
        let journal = Journal::find()
          .filter(journal::Column::Id.eq(input))
          .one(&*conn)
          .await?;
        assert_eq!(journal, output?);
        Ok(())
      }))),
    }),
    Task::FindPage(Input {
      name: "Find Journal Page".to_owned(),
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
          assert!(
            JournalService::contain_user(
              &*conn,
              &journal::Model::from(a.item.clone()),
              auth_user.id().parse::<uuid::Uuid>()?,
              None
            )
            .await?
          );
          assert!(a.item.name < b.item.name);
        }
        Ok(())
      }))),
    }),
    Task::Handle(Input {
      name: "Create Journal".to_owned(),
      auth_user: Arc::new(Box::new(|conn| Box::pin(async move {
        let user = User::find()
          .filter(user::Column::Role.eq(user::Role::User))
          .one(&*conn)
          .await?
          .unwrap();
        Ok(AuthUser::User(user))
      }))),
      input: Arc::new(Box::new(|_| Box::pin(async move {
        Ok(JournalCommand::Create(JournalCommandCreate {
          target_id: None,
          name: "new journal name".to_owned(),
          description: "new journal description".to_owned(),
          unit: "UNIT".to_owned(),
          tags: HashSet::from_iter(vec!["new tag 1".to_owned(), "new tag 2".to_owned()]),
          admins: HashSet::default(),
          members: HashSet::default(),
        }))
      }))),
      checker: Arc::new(Box::new(|(conn, auth_user, input, output)| Box::pin(async move {
        match (&*auth_user, input) {
          (AuthUser::User(auth_user), JournalCommand::Create(input)) => {
            let journal = output?.unwrap().into_presentation(&*conn).await?;
            assert_eq!(input.name, journal.name);
            assert_eq!(input.description, journal.description);
            assert_eq!(input.unit, journal.unit);
            assert!(journal.admins.contains(&(AccessItemType::User, auth_user.id)));
            Ok(())
          }
          _ => unreachable!(),
        }
      }))),
    }),
    Task::HandleAll(Input {
      name: "Create, Update and Delete".to_owned(),
      auth_user: Arc::new(Box::new(|conn| Box::pin(async move {
        Ok(AuthUser::User(
          User::find()
            .filter(user::Column::Role.eq(user::Role::User))
            .one(&*conn)
            .await?
            .unwrap(),
        ))
      }))),
      input: Arc::new(Box::new(|_| Box::pin(async move {
        let lid = uuid::Uuid::new_v4();
        Ok(vec![
          JournalCommand::Create(JournalCommandCreate {
            target_id: Some(lid),
            name: "new journal name".to_owned(),
            description: "new journal description".to_owned(),
            unit: "UNIT".to_owned(),
            tags: HashSet::from_iter(vec!["new tag 1".to_owned(), "new tag 2".to_owned()]),
            admins: HashSet::default(),
            members: HashSet::default(),
          }),
          JournalCommand::Update(JournalCommandUpdate {
            target_id: lid,
            name: Some("new new journal name".to_owned()),
            description: Some("new description".to_owned()),
            unit: Some("NEW_UNIT".to_owned()),
            tags: None,
            is_archived: None,
            admins: None,
            members: None,
          }),
          JournalCommand::Delete(lid),
        ])
      }))),
      checker: Arc::new(Box::new(|(_, auth_user, input, output)| Box::pin(async move {
        match (&*auth_user, &*input, &*output?) {
          (
            AuthUser::User(auth_user),
            &[JournalCommand::Create(ref create), JournalCommand::Update(ref update), JournalCommand::Delete(_)],
            &[Some(ref create_result), Some(ref update_result), None],
          ) => {
            assert_eq!(create.name, create_result.name);
            assert_eq!(create.description, create_result.description);
            assert_eq!(create.unit, create_result.unit);
            assert!(!create_result.is_archived);
            assert!(create_result.admins.contains(&(AccessItemType::User, auth_user.id)));

            assert_eq!(update.name.clone().unwrap(), update_result.name);
            assert_eq!(update.description.clone().unwrap(), update_result.description);
            assert_eq!(update.unit.clone().unwrap(), update_result.unit);
            assert!(!update_result.is_archived);
            assert!(update_result.admins.contains(&(AccessItemType::User, auth_user.id)));

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
  use backend_shared::services::JournalService;

  #[tokio::test]
  async fn test_tasks() -> backend_shared::Result<()> {
    crate::tests::run_test::<JournalService>(&TASKS).await
  }
}
