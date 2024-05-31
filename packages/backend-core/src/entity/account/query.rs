use crate::entity::account::Type;
use crate::entity::{account, account_tag};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Cond, Func, IntoCondition};
use sea_orm::{Condition, QuerySelect, QueryTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Query {
  #[serde(default)]
  pub id: HashSet<Uuid>,
  #[serde(default)]
  pub journal_id: HashSet<Uuid>,
  #[serde(default)]
  pub name: HashSet<String>,
  #[serde(default)]
  pub unit: String,
  #[serde(default)]
  #[serde(rename = "type")]
  pub typ: Option<Type>,
  #[serde(default)]
  pub tags: HashSet<String>,
  #[serde(default)]
  pub full_text: String,
}

impl IntoCondition for Query {
  fn into_condition(self) -> Condition {
    let mut cond = Cond::all();

    if !self.id.is_empty() {
      cond = cond.add(account::Column::Id.is_in(self.id));
    }

    let name: HashSet<String> = self
      .name
      .into_iter()
      .map(|name| name.trim().to_string())
      .filter(|name| !name.is_empty())
      .collect();
    if !name.is_empty() {
      cond = cond.add(account::Column::Name.is_in(name));
    }

    let unit = self.unit.trim().to_string();
    if !unit.is_empty() {
      cond = cond.add(account::Column::Unit.eq(unit));
    }

    if let Some(typ) = self.typ {
      cond = cond.add(account::Column::Typ.eq(typ));
    }

    if !self.journal_id.is_empty() {
      cond = cond.add(account::Column::JournalId.is_in(self.journal_id));
    }

    let keyword = self.full_text.trim().to_lowercase();
    if !keyword.is_empty() {
      let keyword = format!("%{}%", keyword);
      let sub_cond = Cond::any()
        .add(
          Expr::expr(Func::lower(Expr::col((account::Entity, account::Column::Name))))
            .like(keyword.clone()),
        )
        .add(
          Expr::expr(Func::lower(Expr::col((account::Entity, account::Column::Description))))
            .like(keyword.clone()),
        )
        .add(
          account::Column::Id.in_subquery(
            account_tag::Entity::find()
              .select_only()
              .distinct()
              .column(account_tag::Column::AccountId)
              .filter(
                Expr::expr(Func::lower(Expr::col((account_tag::Entity, account_tag::Column::Tag))))
                  .like(keyword.clone()),
              )
              .into_query(),
          ),
        );

      cond = cond.add(sub_cond);
    }

    cond
  }
}

#[cfg(test)]
mod tests {
  use crate::entity::account::{self, Type};
  use sea_orm::{DbBackend, EntityTrait, QueryFilter, QueryTrait};
  use std::collections::HashSet;
  use uuid::uuid;

  #[test]
  fn test_query() -> anyhow::Result<()> {
    let query = account::Query {
      id: HashSet::from_iter([uuid!("50a1b556-b99d-4ae0-bfba-d117f9a958de")]),
      name: HashSet::from_iter(["Name 1".to_string(), "".to_string(), "  ".to_string()]),
      unit: "Unit 1".to_string(),
      typ: Some(Type::Asset),
      journal_id: HashSet::from_iter([uuid!("50a1b556-b99d-4ae0-bfba-d117f9a958de")]),
      tags: HashSet::default(),
      full_text: "Keyword  ".to_string(),
    };

    assert_eq!(
      [r#"SELECT "accounts"."id", "accounts"."journal_id", "accounts"."name", "accounts"."description", "accounts"."unit", "accounts"."type" FROM "accounts""#,
        r#"WHERE "accounts"."id" IN ('50a1b556-b99d-4ae0-bfba-d117f9a958de')"#,
        r#"AND "accounts"."name" IN ('Name 1') AND "accounts"."unit" = 'Unit 1'"#,
        r#"AND "accounts"."type" = 'A' AND "accounts"."journal_id" IN ('50a1b556-b99d-4ae0-bfba-d117f9a958de')"#,
        r#"AND (LOWER("accounts"."name") LIKE '%keyword%' OR LOWER("accounts"."description") LIKE '%keyword%'"#,
        r#"OR "accounts"."id" IN (SELECT DISTINCT "account_tags"."account_id" FROM "account_tags" WHERE LOWER("account_tags"."tag") LIKE '%keyword%'))"#].join(" "),
      account::Entity::find().filter(query).build(DbBackend::Sqlite).to_string()
    );

    Ok(())
  }
}
