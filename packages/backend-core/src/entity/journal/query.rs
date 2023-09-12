use crate::entity::{journal, journal_tag, FIELD_DESCRIPTION, FIELD_NAME, FIELD_TAGS};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Cond, Func, IntoCondition};
use sea_orm::{Condition, QuerySelect, QueryTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Query {
  pub id: HashSet<Uuid>,
  pub name: HashSet<String>,
  pub unit: String,
  pub full_text: (String, HashSet<String>),
}

impl IntoCondition for Query {
  fn into_condition(self) -> Condition {
    let mut cond = Cond::all();

    if !self.id.is_empty() {
      cond = cond.add(journal::Column::Id.is_in(self.id));
    }

    let name: HashSet<String> = self
      .name
      .into_iter()
      .map(|name| name.trim().to_string())
      .filter(|name| !name.is_empty())
      .collect();
    if !name.is_empty() {
      cond = cond.add(journal::Column::Name.is_in(name));
    }

    let unit = self.unit.trim().to_string();
    if !unit.is_empty() {
      cond = cond.add(journal::Column::Unit.eq(unit));
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
          Expr::expr(Func::lower(Expr::col((journal::Entity, journal::Column::Name))))
            .like(keyword.clone()),
        );
      }

      if fields.contains(FIELD_DESCRIPTION) {
        sub_cond = sub_cond.add(
          Expr::expr(Func::lower(Expr::col((journal::Entity, journal::Column::Description))))
            .like(keyword.clone()),
        );
      }

      if fields.contains(FIELD_TAGS) {
        sub_cond = sub_cond.add(
          journal::Column::Id.in_subquery(
            journal_tag::Entity::find()
              .select_only()
              .distinct()
              .column(journal_tag::Column::JournalId)
              .filter(
                Expr::expr(Func::lower(Expr::col((journal_tag::Entity, journal_tag::Column::Tag))))
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
  use crate::entity::{journal, FIELD_DESCRIPTION, FIELD_NAME, FIELD_TAGS};
  use sea_orm::{DbBackend, EntityTrait, QueryFilter, QueryTrait};
  use std::collections::HashSet;
  use uuid::uuid;

  #[test]
  fn test_query() -> anyhow::Result<()> {
    let query = journal::Query {
      id: HashSet::from_iter([uuid!("50a1b556-b99d-4ae0-bfba-d117f9a958de")]),
      name: HashSet::from_iter(["Name 1".to_string(), "".to_string(), "  ".to_string()]),
      unit: "Unit 1".to_string(),
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
      [r#"SELECT "journal"."id", "journal"."name", "journal"."description", "journal"."unit" FROM "journal""#,
        r#"WHERE "journal"."id" IN ('50a1b556-b99d-4ae0-bfba-d117f9a958de')"#,
        r#"AND "journal"."name" IN ('Name 1') AND "journal"."unit" = 'Unit 1'"#,
        r#"AND (LOWER("journal"."name") LIKE '%keyword%' OR LOWER("journal"."description") LIKE '%keyword%'"#,
        r#"OR "journal"."id" IN (SELECT DISTINCT "journal_tag"."journal_id" FROM "journal_tag" WHERE LOWER("journal_tag"."tag") LIKE '%keyword%'))"#].join(" "),
      journal::Entity::find().filter(query).build(DbBackend::Sqlite).to_string()
    );

    Ok(())
  }
}
