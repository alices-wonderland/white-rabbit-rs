pub mod account;
pub mod account_tag;
pub mod auth_id;
pub mod group;
pub mod group_user;
pub mod journal;
pub mod journal_group;
pub mod journal_tag;
pub mod journal_user;
pub mod record;
pub mod record_item;
pub mod record_tag;
pub mod user;

use serde::{Deserialize, Serialize};

pub use account::Entity as Account;
pub use account_tag::Entity as AccountTag;
pub use auth_id::Entity as AuthId;
pub use group::Entity as Group;
pub use group_user::Entity as GroupUser;
pub use journal::Entity as Journal;
pub use journal_group::Entity as JournalGroup;
pub use journal_tag::Entity as JournalTag;
pub use journal_user::Entity as JournalUser;
pub use record::Entity as Record;
pub use record_item::Entity as RecordItem;
pub use record_tag::Entity as RecordTag;
pub use user::Entity as User;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum_macros::Display, Serialize, Deserialize)]
pub enum AccessItemType {
  User,
  Group,
}
