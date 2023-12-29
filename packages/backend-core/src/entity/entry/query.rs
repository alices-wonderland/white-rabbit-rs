use crate::entity::entry::Type;
use crate::entity::{entry, entry_item, entry_tag, FIELD_DESCRIPTION, FIELD_NAME, FIELD_TAGS};
use chrono::NaiveDate;
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
  pub account_id: HashSet<Uuid>,
  #[serde(default)]
  pub name: HashSet<String>,
  #[serde(default)]
  #[serde(rename = "type")]
  pub typ: Option<Type>,
  #[serde(default)]
  pub start: Option<NaiveDate>,
  #[serde(default)]
  pub end: Option<NaiveDate>,
  #[serde(default)]
  pub full_text: (String, HashSet<String>),
}

impl IntoCondition for Query {
  fn into_condition(self) -> Condition {
    let mut cond = Cond::all();

    if !self.id.is_empty() {
      cond = cond.add(entry::Column::Id.is_in(self.id));
    }

    if !self.journal_id.is_empty() {
      cond = cond.add(entry::Column::JournalId.is_in(self.journal_id));
    }

    if !self.account_id.is_empty() {
      cond = cond.add(
        entry::Column::Id.in_subquery(
          entry_item::Entity::find()
            .select_only()
            .distinct()
            .column(entry_item::Column::EntryId)
            .filter(entry_item::Column::AccountId.is_in(self.account_id))
            .into_query(),
        ),
      );
    }

    let name: HashSet<String> = self
      .name
      .into_iter()
      .map(|name| name.trim().to_string())
      .filter(|name| !name.is_empty())
      .collect();
    if !name.is_empty() {
      cond = cond.add(entry::Column::Name.is_in(name));
    }

    if let Some(typ) = self.typ {
      cond = cond.add(entry::Column::Typ.eq(typ));
    }

    if let Some(start) = self.start {
      cond = cond.add(entry::Column::Date.gte(start));
    }

    if let Some(end) = self.end {
      cond = cond.add(entry::Column::Date.lte(end));
    }

    let keyword = self.full_text.0.trim().to_lowercase();
    if !keyword.is_empty() {
      let keyword = format!("%{}%", keyword);
      let mut sub_cond = Cond::any();
      let fields = if self.full_text.1.is_empty() {
        HashSet::from_iter([
          FIELD_NAME.to_string(),
          FIELD_TAGS.to_string(),
          FIELD_DESCRIPTION.to_string(),
        ])
      } else {
        self.full_text.1
      };
      if fields.contains(FIELD_NAME) {
        sub_cond = sub_cond.add(
          Expr::expr(Func::lower(Expr::col((entry::Entity, entry::Column::Name))))
            .like(keyword.clone()),
        );
      }

      if fields.contains(FIELD_DESCRIPTION) {
        sub_cond = sub_cond.add(
          Expr::expr(Func::lower(Expr::col((entry::Entity, entry::Column::Description))))
            .like(keyword.clone()),
        );
      }

      if fields.contains(FIELD_TAGS) {
        sub_cond = sub_cond.add(
          entry::Column::Id.in_subquery(
            entry_tag::Entity::find()
              .select_only()
              .distinct()
              .column(entry_tag::Column::EntryId)
              .filter(
                Expr::expr(Func::lower(Expr::col((entry_tag::Entity, entry_tag::Column::Tag))))
                  .like(keyword.clone()),
              )
              .into_query(),
          ),
        );
      }

      if !sub_cond.is_empty() {
        cond = cond.add(sub_cond);
      }
    }

    cond
  }
}

#[cfg(test)]
mod tests {
  use crate::entity::entry::Type;
  use crate::entity::{entry, FIELD_DESCRIPTION, FIELD_NAME, FIELD_TAGS};
  use chrono::NaiveDate;
  use sea_orm::{DbBackend, EntityTrait, QueryFilter, QueryTrait};
  use std::collections::HashSet;
  use uuid::uuid;

  #[test]
  fn test_query() -> anyhow::Result<()> {
    let query = entry::Query {
      id: HashSet::from_iter([uuid!("50a1b556-b99d-4ae0-bfba-d117f9a958de")]),
      journal_id: HashSet::from_iter([uuid!("50a1b556-b99d-4ae0-bfba-d117f9a958de")]),
      account_id: HashSet::from_iter([uuid!("50a1b556-b99d-4ae0-bfba-d117f9a958de")]),
      name: HashSet::from_iter(["Name 1".to_string(), "".to_string(), "  ".to_string()]),
      typ: Some(Type::Check),
      start: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
      end: Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
      full_text: (
        "Keyword  ".to_string(),
        HashSet::from_iter([
          FIELD_NAME.to_string(),
          FIELD_DESCRIPTION.to_string(),
          FIELD_TAGS.to_string(),
        ]),
      ),
    };

    assert_eq!(
      [r#"SELECT "entries"."id", "entries"."journal_id", "entries"."name", "entries"."description", "entries"."type", "entries"."date" FROM "entries""#,
        r#"WHERE "entries"."id" IN ('50a1b556-b99d-4ae0-bfba-d117f9a958de')"#,
        r#"AND "entries"."journal_id" IN ('50a1b556-b99d-4ae0-bfba-d117f9a958de')"#,
        r#"AND "entries"."id" IN (SELECT DISTINCT "entry_items"."entry_id" FROM "entry_items" WHERE "entry_items"."account_id" IN ('50a1b556-b99d-4ae0-bfba-d117f9a958de'))"#,
        r#"AND "entries"."name" IN ('Name 1') AND "entries"."type" = 'C' AND "entries"."date" >= '2023-01-01' AND "entries"."date" <= '2023-12-31'"#,
        r#"AND (LOWER("entries"."name") LIKE '%keyword%' OR LOWER("entries"."description") LIKE '%keyword%'"#,
        r#"OR "entries"."id" IN (SELECT DISTINCT "entry_tags"."entry_id" FROM "entry_tags" WHERE LOWER("entry_tags"."tag") LIKE '%keyword%'))"#].join(" "),
      entry::Entity::find().filter(query).build(DbBackend::Sqlite).to_string()
    );

    Ok(())
  }
}
