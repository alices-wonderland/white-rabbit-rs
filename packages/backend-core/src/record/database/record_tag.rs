use crate::record;
use sea_orm::entity::prelude::*;

use uuid::Uuid;

#[derive(Clone, Debug, Hash, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "record_tag")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub record_id: Uuid,
  #[sea_orm(primary_key)]
  pub tag: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "record::Entity",
    from = "Column::RecordId",
    to = "crate::record::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Record,
}

impl Related<record::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Record.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
