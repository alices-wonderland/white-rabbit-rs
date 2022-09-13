use futures::stream::{self, StreamExt};
use sea_orm::sea_query::{IntoCondition, Value};
use sea_orm::{
  ColumnTrait, Condition, ConnectionTrait, EntityTrait, FromQueryResult, ModelTrait, PrimaryKeyToColumn,
  PrimaryKeyTrait, QueryFilter, QueryOrder, QuerySelect, Select,
};
use serde::{Deserialize, Serialize};

use super::AuthUser;

const DEFAULT_SIZE: usize = 100;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindAllInput<Q>
where
  Q: Sized + IntoCondition,
{
  pub query: Option<Q>,
  pub size: Option<usize>,
  pub sort: Option<Sort>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindPageInput<Q>
where
  Q: Sized + IntoCondition,
{
  pub query: Option<Q>,
  pub pagination: Option<Pagination>,
  pub sort: Sort,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page<M> {
  pub info: PageInfo,
  pub items: Vec<PageItem<M>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageInfo {
  pub has_previous: bool,
  pub has_next: bool,
  pub start_cursor: Option<String>,
  pub end_cursor: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageItem<M> {
  pub cursor: String,
  pub item: M,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pagination {
  pub after: Option<String>,
  pub before: Option<String>,
  pub size: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sort {
  pub field: String,
  pub order: Order,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Order {
  Asc,
  Desc,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IdQuery {
  Single(uuid::Uuid),
  Multiple(Vec<uuid::Uuid>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextQuery {
  Value(String),
  FullText(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullTextQuery {
  pub value: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub fields: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainingUserQuery {
  Value(uuid::Uuid),
  Object {
    id: uuid::Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    fields: Option<Vec<String>>,
  },
}

impl ContainingUserQuery {
  pub fn id(&self) -> uuid::Uuid {
    match self {
      ContainingUserQuery::Value(id) => *id,
      ContainingUserQuery::Object { id, .. } => *id,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExternalQuery {
  ContainingUser(ContainingUserQuery),
  FullText(FullTextQuery),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeQuery<E: Sized> {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  #[serde(rename = "__gt")]
  pub gt: Option<E>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  #[serde(rename = "__lt")]
  pub lt: Option<E>,
}

#[async_trait::async_trait]
pub trait AbstractReadService {
  type Model: ModelTrait<Entity = Self::Entity> + FromQueryResult + Sync + Send;

  type Entity: EntityTrait<Model = Self::Model, PrimaryKey = Self::PrimaryKey>;

  type PrimaryKey: PrimaryKeyTrait<ValueType = uuid::Uuid> + PrimaryKeyToColumn;

  type Query: IntoCondition + Into<Vec<ExternalQuery>> + Clone + Sync + Send;

  async fn filter_by_external_query(items: Vec<Self::Model>, external_query: &ExternalQuery) -> Vec<Self::Model>;

  fn find_all_select() -> Select<Self::Entity>;

  fn primary_field() -> <Self::Entity as EntityTrait>::Column;

  fn primary_value(model: &Self::Model) -> uuid::Uuid;

  fn sortable_field(field: &str) -> Option<<Self::Entity as EntityTrait>::Column>;

  fn sortable_value(field: &str, model: &Self::Model) -> Option<Value> {
    Self::sortable_field(field).map(|col| model.get(col))
  }

  async fn find_by_cursor(conn: &impl ConnectionTrait, cursor: Option<String>) -> anyhow::Result<Option<Self::Model>> {
    if let Some(cursor) = cursor {
      let id: uuid::Uuid = String::from_utf8(base64::decode(&cursor)?)?.parse()?;
      Ok(Self::Entity::find_by_id(id).one(conn).await?)
    } else {
      Ok(None)
    }
  }

  fn encode_cursor(model: &Self::Model) -> String {
    let id = Self::primary_value(model).to_string();
    base64::encode(id)
  }

  async fn is_readable(_conn: &impl ConnectionTrait, _operator: &AuthUser, _model: &Self::Model) -> bool {
    true
  }

  async fn filter_by_external_queries(items: Vec<Self::Model>, external_queries: &[ExternalQuery]) -> Vec<Self::Model> {
    let mut result = items;
    for query in external_queries {
      result = Self::filter_by_external_query(result, query).await;
      if result.is_empty() {
        break;
      }
    }
    result
  }

  async fn do_find_all(
    conn: &impl ConnectionTrait,
    operator: &AuthUser,
    condition: Condition,
    external_queries: &[ExternalQuery],
    sort: Option<Sort>,
    size: usize,
  ) -> Result<Vec<Self::Model>, sea_orm::DbErr> {
    let mut results = Vec::new();
    let mut count = 0;
    while results.len() < size as usize {
      let mut statement: Select<Self::Entity> = Self::find_all_select().filter(condition.clone());

      if let Some(ref item) = sort {
        if let Some(column) = Self::sortable_field(&item.field) {
          statement = statement.order_by(
            column,
            match item.order {
              Order::Asc => sea_orm::sea_query::Order::Asc,
              Order::Desc => sea_orm::sea_query::Order::Desc,
            },
          );
        }
      }

      let result = statement
        .offset(count * (size as u64))
        .limit(size as u64)
        .all(conn)
        .await?;
      if result.is_empty() {
        break;
      }
      count += 1;
      let result = stream::iter(result)
        .filter_map(|item| async move {
          if Self::is_readable(conn, operator, &item).await {
            Some(item)
          } else {
            None
          }
        })
        .collect()
        .await;
      let mut result = Self::filter_by_external_queries(result, external_queries).await;
      results.append(&mut result);
    }

    Ok(results)
  }

  async fn find_by_id(
    conn: &impl ConnectionTrait,
    operator: AuthUser,
    id: uuid::Uuid,
  ) -> anyhow::Result<Option<Self::Model>> {
    if let Some(model) = Self::Entity::find_by_id(id).one(conn).await? {
      if Self::is_readable(conn, &operator, &model).await {
        return Ok(Some(model));
      }
    }
    Ok(None)
  }

  async fn find_all(
    conn: &impl ConnectionTrait,
    operator: AuthUser,
    input: FindAllInput<Self::Query>,
  ) -> anyhow::Result<Vec<Self::Model>> {
    let condition = input
      .query
      .as_ref()
      .map(|query| query.clone().into_condition())
      .unwrap_or_else(Condition::all);

    let external_queries: Vec<ExternalQuery> = input.query.map(|query| query.into()).unwrap_or_default();

    Ok(
      Self::do_find_all(
        conn,
        &operator,
        condition,
        &external_queries,
        input.sort,
        input.size.unwrap_or(DEFAULT_SIZE),
      )
      .await?,
    )
  }

  async fn find_page(
    conn: &impl ConnectionTrait,
    operator: AuthUser,
    input: FindPageInput<Self::Query>,
  ) -> anyhow::Result<Page<Self::Model>> {
    let mut condition = input
      .query
      .as_ref()
      .map(|query| query.clone().into_condition())
      .unwrap_or_else(Condition::all);

    let external_queries: Vec<ExternalQuery> = input.query.map(|query| query.into()).unwrap_or_default();

    let mut sort = input.sort;
    let mut is_reversed = false;
    let size = input.pagination.as_ref().and_then(|p| p.size).unwrap_or(DEFAULT_SIZE);
    let mut after_model = None;
    let mut before_model = None;

    if let Some(Pagination { after, before, .. }) = &input.pagination {
      after_model = Self::find_by_cursor(conn, after.clone()).await?;
      before_model = Self::find_by_cursor(conn, before.clone()).await?;

      if before_model.is_some() && after_model.is_none() {
        is_reversed = true;
      }

      if let Some(column) = Self::sortable_field(&sort.field) {
        if let Some((model, value)) = after_model
          .as_ref()
          .and_then(|m| Self::sortable_value(&sort.field, m).map(|v| (m, v)))
        {
          let value_eq_cond = {
            Condition::all().add(column.eq(value.clone())).add(if is_reversed {
              Self::primary_field().lt(Self::primary_value(model))
            } else {
              Self::primary_field().gt(Self::primary_value(model))
            })
          };

          condition = condition.add(match sort.order {
            Order::Asc => Condition::any().add(column.gt(value)).add(value_eq_cond),
            Order::Desc => Condition::any().add(column.lt(value)).add(value_eq_cond),
          });
        }

        if let Some((model, value)) = before_model
          .as_ref()
          .and_then(|m| Self::sortable_value(&sort.field, m).map(|v| (m, v)))
        {
          let value_eq_cond = {
            Condition::all().add(column.eq(value.clone())).add(if is_reversed {
              Self::primary_field().gt(Self::primary_value(model))
            } else {
              Self::primary_field().lt(Self::primary_value(model))
            })
          };

          condition = condition.add(match sort.order {
            Order::Asc => Condition::any().add(column.lt(value)).add(value_eq_cond),
            Order::Desc => Condition::any().add(column.gt(value)).add(value_eq_cond),
          });
        }
      }

      if is_reversed {
        sort = Sort {
          order: match sort.order {
            Order::Asc => Order::Desc,
            Order::Desc => Order::Asc,
          },
          ..sort
        };
      }
    }

    let mut result = Self::do_find_all(conn, &operator, condition, &external_queries, Some(sort), size + 1).await?;
    let has_next = result.len() > size;
    let has_previous = (is_reversed && before_model.is_some()) || (!is_reversed && after_model.is_some());

    if is_reversed {
      result.reverse();
    }

    let result: Vec<_> = result
      .into_iter()
      .take(size)
      .map(|item| PageItem {
        cursor: Self::encode_cursor(&item),
        item,
      })
      .collect();

    Ok(Page {
      info: PageInfo {
        has_previous,
        has_next,
        start_cursor: result.first().map(|item| item.cursor.clone()),
        end_cursor: result.last().map(|item| item.cursor.clone()),
      },
      items: result,
    })
  }
}

#[cfg(test)]
mod tests {

  use crate::models::{user, User};
  use crate::services::read_service::{AbstractReadService, FindPageInput, Order, Pagination, Sort};
  use crate::services::user::UserService;
  use crate::services::AuthUser;
  use crate::{run, services::read_service::FullTextQuery};

  use super::{IdQuery, RangeQuery, TextQuery};
  use chrono::{TimeZone, Utc};
  use migration::{Migrator, MigratorTrait};
  use sea_orm::{EntityTrait, Set};
  use serde_json::json;

  #[tokio::test]
  async fn test_full_text_query() -> anyhow::Result<()> {
    dotenv::from_filename(".test.env")?;
    let _ = env_logger::try_init();

    let db = run().await?;
    Migrator::up(&db, None).await?;

    let user_ids = vec![
      uuid::Uuid::new_v4(),
      uuid::Uuid::new_v4(),
      uuid::Uuid::new_v4(),
      uuid::Uuid::new_v4(),
      uuid::Uuid::new_v4(),
    ];

    let users = (0..5)
      .map(|idx| user::ActiveModel {
        id: Set(user_ids[idx]),
        name: Set(format!("User {}", idx)),
        role: Set(match idx % 3 {
          0 => user::Role::Admin,
          1 => user::Role::User,
          _ => user::Role::Owner,
        }),
      })
      .collect::<Vec<_>>();

    let _ = User::insert_many(users).exec(&db).await?;
    let users = User::find().all(&db).await?;
    log::info!("Users: {:#?}", users);

    let page = UserService::find_page(
      &db,
      AuthUser::Id(("Provider".to_owned(), "Value".to_owned())),
      FindPageInput {
        sort: Sort {
          field: "name".to_owned(),
          order: Order::Asc,
        },
        pagination: Some(Pagination {
          size: Some(3),
          ..Default::default()
        }),
        query: None,
      },
    )
    .await?;
    log::info!("page 1: {:#?}", page);
    assert_eq!(page.items.len(), 3);
    assert_eq!(page.items[0].item.id, user_ids[0]);
    assert_eq!(page.items[2].item.id, user_ids[2]);

    let page = UserService::find_page(
      &db,
      AuthUser::Id(("Provider".to_owned(), "Value".to_owned())),
      FindPageInput {
        sort: Sort {
          field: "name".to_owned(),
          order: Order::Asc,
        },
        pagination: Some(Pagination {
          size: Some(3),
          after: page.info.end_cursor,
          ..Default::default()
        }),
        query: None,
      },
    )
    .await?;
    log::info!("page 2: {:#?}", page);
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].item.id, user_ids[3]);
    assert_eq!(page.items[1].item.id, user_ids[4]);

    let page = UserService::find_page(
      &db,
      AuthUser::Id(("Provider".to_owned(), "Value".to_owned())),
      FindPageInput {
        sort: Sort {
          field: "name".to_owned(),
          order: Order::Asc,
        },
        pagination: Some(Pagination {
          size: Some(3),
          before: page.info.start_cursor,
          ..Default::default()
        }),
        query: None,
      },
    )
    .await?;
    log::info!("page 3: {:#?}", page);
    assert_eq!(page.items.len(), 3);
    assert_eq!(page.items[0].item.id, user_ids[0]);
    assert_eq!(page.items[2].item.id, user_ids[2]);

    let page = UserService::find_page(
      &db,
      AuthUser::Id(("Provider".to_owned(), "Value".to_owned())),
      FindPageInput {
        sort: Sort {
          field: "name".to_owned(),
          order: Order::Desc,
        },
        pagination: Some(Pagination {
          size: Some(3),
          ..Default::default()
        }),
        query: None,
      },
    )
    .await?;
    log::info!("page 4: {:#?}", page);
    assert_eq!(page.items.len(), 3);
    assert_eq!(page.items[0].item.id, user_ids[4]);
    assert_eq!(page.items[2].item.id, user_ids[2]);
    assert!(page.info.end_cursor.is_some());
    assert!(page.info.start_cursor.is_some());

    let page = UserService::find_page(
      &db,
      AuthUser::Id(("Provider".to_owned(), "Value".to_owned())),
      FindPageInput {
        sort: Sort {
          field: "name".to_owned(),
          order: Order::Desc,
        },
        pagination: Some(Pagination {
          size: Some(3),
          after: page.info.end_cursor,
          ..Default::default()
        }),
        query: None,
      },
    )
    .await?;
    log::info!("page 5: {:#?}", page);
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].item.id, user_ids[1]);
    assert_eq!(page.items[1].item.id, user_ids[0]);

    let page = UserService::find_page(
      &db,
      AuthUser::Id(("Provider".to_owned(), "Value".to_owned())),
      FindPageInput {
        sort: Sort {
          field: "name".to_owned(),
          order: Order::Desc,
        },
        pagination: Some(Pagination {
          size: Some(4),
          before: page.info.start_cursor,
          ..Default::default()
        }),
        query: None,
      },
    )
    .await?;
    log::info!("page 6: {:#?}", page);
    assert_eq!(page.items.len(), 3);
    assert_eq!(page.items[0].item.id, user_ids[4]);
    assert_eq!(page.items[2].item.id, user_ids[2]);
    assert!(page.info.end_cursor.is_some());
    assert!(page.info.start_cursor.is_some());

    Migrator::down(&db, None).await?;
    Ok(())
  }

  #[test]
  fn test_serde() -> anyhow::Result<()> {
    dotenv::from_filename(".test.env")?;
    let _ = env_logger::try_init();

    let id = uuid::Uuid::new_v4();
    let query = IdQuery::Single(id);
    let json = serde_json::to_value(query.clone())?;
    assert_eq!(json, json!(id));
    assert_eq!(query, serde_json::from_value::<IdQuery>(json)?);

    let id = uuid::Uuid::new_v4();
    let query = IdQuery::Multiple(vec![id]);
    let json = serde_json::to_value(query.clone())?;
    assert_eq!(json, json!([id]));
    assert_eq!(query, serde_json::from_value::<IdQuery>(json)?);

    let query = TextQuery::Value("String".to_owned());
    let json = serde_json::to_value(query.clone())?;
    assert_eq!(json, json!("String"));
    assert_eq!(query, serde_json::from_value::<TextQuery>(json)?);

    let query = FullTextQuery {
      value: "FullText".to_owned(),
      fields: Some(vec!["field1".to_owned(), "field2".to_owned()]),
    };
    let json = serde_json::to_value(query.clone())?;
    assert_eq!(json, json!({ "value": "FullText", "fields": ["field1", "field2"] }));
    assert_eq!(query, serde_json::from_value::<FullTextQuery>(json)?);

    let query = RangeQuery {
      gt: Some(Utc.ymd(2022, 1, 1).and_hms(0, 0, 0)),
      lt: Some(Utc.ymd(2023, 1, 1).and_hms(0, 0, 0)),
    };
    let json = serde_json::to_value(query.clone())?;
    assert_eq!(
      json,
      json!({ "__gt": "2022-01-01T00:00:00Z", "__lt": "2023-01-01T00:00:00Z" })
    );
    assert_eq!(query, serde_json::from_value::<RangeQuery<_>>(json)?);

    let query = RangeQuery {
      gt: Some(10),
      lt: Some(42),
    };
    let json = serde_json::to_value(query.clone())?;
    assert_eq!(json, json!({ "__gt": 10, "__lt": 42 }));
    assert_eq!(query, serde_json::from_value::<RangeQuery<_>>(json)?);

    let query = RangeQuery {
      gt: Some(10.1),
      lt: None,
    };
    let json = serde_json::to_value(query.clone())?;
    assert_eq!(json, json!({ "__gt": 10.1 }));
    assert_eq!(query, serde_json::from_value::<RangeQuery<_>>(json)?);

    Ok(())
  }
}
