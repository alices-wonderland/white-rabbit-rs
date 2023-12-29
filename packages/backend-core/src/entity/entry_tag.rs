use crate::entity::entry;
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, Hash, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "entry_tags")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub entry_id: Uuid,
  #[sea_orm(primary_key)]
  pub tag: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "entry::Entity",
    from = "Column::EntryId",
    to = "entry::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Entry,
}

impl Related<entry::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Entry.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
