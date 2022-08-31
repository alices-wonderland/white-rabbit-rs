use crate::models::user::Model as UserModel;
use sea_orm::sea_query::IntoCondition;
use sea_orm::{ConnectionTrait, PrimaryKeyTrait, QueryFilter, QueryOrder, QuerySelect, Select};
use sea_orm::{EntityTrait, FromQueryResult, ModelTrait};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindAllInput<Q>
where
  Q: Sized + IntoCondition,
{
  pub query: Option<Q>,
  pub pagination: Option<Pagination>,
  pub sort: Option<Sort>,
}

type Sort = Vec<SortItem>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pagination {
  pub after: Option<String>,
  pub before: Option<String>,
  pub size: Option<usize>,
  pub offset: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortItem {
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
  Single(i32),
  Multiple(Vec<i32>),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExternalQuery {
  ContainUser(i32),
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

  type PrimaryKey: PrimaryKeyTrait<ValueType = i32>;

  type Query: IntoCondition + Into<Vec<ExternalQuery>> + Clone + Sync + Send;

  async fn filter_by_external_query(items: Vec<Self::Model>, external_query: &ExternalQuery) -> Vec<Self::Model>;

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

  fn find_all_select() -> Select<Self::Entity>;

  fn sortable_field(field: &str) -> Option<<Self::Entity as EntityTrait>::Column>;

  async fn do_find_all(
    conn: &impl ConnectionTrait,
    query: Option<Self::Query>,
    sort: Sort,
    size: u64,
  ) -> Result<Vec<Self::Model>, sea_orm::DbErr> {
    let mut results = Vec::new();
    let query = query.map(|q| (q.clone().into_condition(), q.into()));
    let mut count = 0;
    while results.len() < size as usize {
      let mut statement: Select<Self::Entity> = match query {
        Some((ref cond, _)) => Self::find_all_select().filter(cond.clone()),
        None => Self::find_all_select(),
      };

      for item in &sort {
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

      let result = statement.offset(count * size).limit(size).all(conn).await?;
      if result.is_empty() {
        break;
      }
      count += 1;
      let mut result = match query {
        Some((_, ref external)) => Self::filter_by_external_queries(result, external).await,
        None => result,
      };
      results.append(&mut result);
    }

    Ok(results)
  }

  async fn is_readable(
    &self,
    _conn: &impl ConnectionTrait,
    _operator: Option<&UserModel>,
    _model: &Self::Model,
  ) -> bool {
    true
  }

  async fn find_by_id(
    &self,
    conn: &impl ConnectionTrait,
    operator: Option<&UserModel>,
    id: i32,
  ) -> Result<Option<Self::Model>, anyhow::Error> {
    if let Some(model) = Self::Entity::find_by_id(id).one(conn).await? {
      if self.is_readable(conn, operator, &model).await {
        return Ok(Some(model));
      }
    }

    Ok(None)
  }

  async fn find_all(
    &self,
    _conn: &impl ConnectionTrait,
    _operator: Option<&UserModel>,
    _input: FindAllInput<Self::Query>,
  ) -> Result<Vec<Self::Model>, anyhow::Error> {
    Err(anyhow::Error::msg("unimplemented"))
  }
}

#[cfg(test)]
mod tests {
  use crate::services::read_service::FullTextQuery;

  use super::{IdQuery, RangeQuery, TextQuery};
  use chrono::{TimeZone, Utc};
  use serde_json::json;
  use std::env;

  #[test]
  fn test_serde() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "info");
    let _ = env_logger::try_init();

    let query = IdQuery::Single(42);
    let json = serde_json::to_value(query.clone())?;
    assert_eq!(json, json!(42));
    assert_eq!(query, serde_json::from_value::<IdQuery>(json)?);

    let query = IdQuery::Multiple(vec![1, 2, 3, 4]);
    let json = serde_json::to_value(query.clone())?;
    assert_eq!(json, json!([1, 2, 3, 4]));
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
