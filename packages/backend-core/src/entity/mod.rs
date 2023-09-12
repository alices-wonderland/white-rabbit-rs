pub mod account;
pub mod account_tag;
pub mod entry;
pub mod entry_item;
pub mod entry_tag;
pub mod journal;
pub mod journal_tag;

pub const FIELD_NAME: &str = "name";
pub const FIELD_DESCRIPTION: &str = "description";
pub const FIELD_TAGS: &str = "tags";
pub const FIELD_TAG_EACH: &str = "tags.each";
pub const FIELD_UNIT: &str = "unit";

pub const MIN_NAME_LENGTH: usize = 6;
pub const MAX_NAME_LENGTH: usize = 63;
pub const MAX_DESCRIPTION_LENGTH: usize = 1023;
pub const MIN_SHORT_TEXT_LENGTH: usize = 2;
pub const MAX_SHORT_TEXT_LENGTH: usize = 15;
pub const MAX_TAGS_LENGTH: usize = 7;
