use futures::stream::{self, StreamExt, TryStreamExt};
use sea_orm::sea_query::{IntoCondition, Value};
use sea_orm::{
  ColumnTrait, Condition, ConnectionTrait, EntityTrait, FromQueryResult, ModelTrait, PrimaryKeyToColumn,
  PrimaryKeyTrait, QueryFilter, QueryOrder, QuerySelect, Select,
};
use serde::{Deserialize, Serialize};

use crate::models::IntoPresentation;

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
  Q: Sized + IntoCondition + Clone,
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

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ComparableQuery<V: Ord + PartialOrd> {
  pub eq: Option<V>,
  pub gt: Option<V>,
  pub lt: Option<V>,
  pub gte: Option<V>,
  pub lte: Option<V>,
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
  type Model: ModelTrait<Entity = Self::Entity>
    + IntoPresentation<Presentation = Self::Presentation>
    + FromQueryResult
    + Sync
    + Send;

  type Entity: EntityTrait<Model = Self::Model, PrimaryKey = Self::PrimaryKey>;

  type Presentation: Send + Sync;

  type PrimaryKey: PrimaryKeyTrait<ValueType = uuid::Uuid> + PrimaryKeyToColumn;

  type Query: IntoCondition + Into<Vec<ExternalQuery>> + Clone + Sync + Send;

  async fn filter_by_external_query(
    conn: &impl ConnectionTrait,
    items: Vec<Self::Model>,
    external_query: &ExternalQuery,
  ) -> Vec<Self::Model>;

  fn find_all_select() -> Select<Self::Entity>;

  fn primary_field() -> <Self::Entity as EntityTrait>::Column;

  fn primary_value(model: &Self::Model) -> uuid::Uuid;

  fn sortable_field(field: &str) -> Option<<Self::Entity as EntityTrait>::Column>;

  async fn is_readable(_conn: &impl ConnectionTrait, _operator: &AuthUser, _model: &Self::Model) -> bool {
    true
  }

  fn sortable_value(field: &str, model: &Self::Model) -> Option<Value> {
    Self::sortable_field(field).map(|col| model.get(col))
  }

  async fn find_by_cursor(conn: &impl ConnectionTrait, cursor: Option<String>) -> crate::Result<Option<Self::Model>> {
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

  async fn filter_by_external_queries(
    conn: &impl ConnectionTrait,
    items: Vec<Self::Model>,
    external_queries: &[ExternalQuery],
  ) -> Vec<Self::Model> {
    let mut result = items;
    for query in external_queries {
      result = Self::filter_by_external_query(conn, result, query).await;
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
      let mut result = Self::filter_by_external_queries(conn, result, external_queries).await;
      results.append(&mut result);
    }

    Ok(results)
  }

  async fn find_by_id(
    conn: &impl ConnectionTrait,
    operator: &AuthUser,
    id: uuid::Uuid,
  ) -> crate::Result<Option<Self::Model>> {
    if let Some(model) = Self::Entity::find_by_id(id).one(conn).await? {
      if Self::is_readable(conn, operator, &model).await {
        return Ok(Some(model));
      }
    }
    Ok(None)
  }

  async fn find_all(
    conn: &impl ConnectionTrait,
    operator: &AuthUser,
    input: FindAllInput<Self::Query>,
  ) -> crate::Result<Vec<Self::Model>> {
    let condition = input
      .query
      .as_ref()
      .map(|query| query.clone().into_condition())
      .unwrap_or_else(Condition::all);

    let external_queries: Vec<ExternalQuery> = input.query.map(|query| query.into()).unwrap_or_default();

    Ok(
      Self::do_find_all(
        conn,
        operator,
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
    operator: &AuthUser,
    input: FindPageInput<Self::Query>,
  ) -> crate::Result<Page<Self::Presentation>> {
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

    let mut result = Self::do_find_all(conn, operator, condition, &external_queries, Some(sort), size + 1).await?;
    let has_next = result.len() > size;
    let has_previous = (is_reversed && before_model.is_some()) || (!is_reversed && after_model.is_some());

    if is_reversed {
      result.reverse();
    }

    let result: Vec<PageItem<Self::Presentation>> = stream::iter(result)
      .take(size)
      .then(|item| async move {
        let cursor = Self::encode_cursor(&item);
        item.into_presentation(conn).await.map(|item| PageItem { cursor, item })
      })
      .try_collect()
      .await?;

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
