use crate::entity::account::{Root, Type, TYPE};
use crate::entity::{
  normalize_description, normalize_name, normalize_tags, normalize_unit, FIELD_JOURNAL, FIELD_TYPE,
};
use crate::error::ErrorRequiredField;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct Builder {
  id: Option<Uuid>,
  journal_id: Option<Uuid>,
  name: String,
  description: String,
  unit: String,
  typ: Option<Type>,
  tags: HashSet<String>,
}

impl From<Root> for Builder {
  fn from(value: Root) -> Self {
    Builder {
      id: Some(value.id),
      journal_id: Some(value.journal_id),
      name: value.name,
      description: value.description,
      unit: value.unit,
      typ: Some(value.typ),
      tags: value.tags,
    }
  }
}

impl Builder {
  pub fn build(self) -> crate::Result<Root> {
    let name = normalize_name(TYPE, self.name)?;
    let description = normalize_description(TYPE, self.description)?;
    let unit = normalize_unit(TYPE, self.unit)?;
    let tags = normalize_tags(TYPE, self.tags)?;
    Ok(Root {
      id: self.id.unwrap_or_else(Uuid::new_v4),
      journal_id: self.journal_id.ok_or_else(|| {
        crate::Error::RequiredField(ErrorRequiredField {
          entity: TYPE.to_string(),
          field: FIELD_JOURNAL.to_string(),
        })
      })?,
      name,
      description,
      unit,
      typ: self.typ.ok_or_else(|| {
        crate::Error::RequiredField(ErrorRequiredField {
          entity: TYPE.to_string(),
          field: FIELD_TYPE.to_string(),
        })
      })?,
      tags,
    })
  }

  pub fn id(self, id: Uuid) -> Builder {
    Builder { id: Some(id), ..self }
  }

  pub fn journal_id(self, journal_id: Uuid) -> Builder {
    Builder { journal_id: Some(journal_id), ..self }
  }

  pub fn name(self, name: impl ToString) -> Builder {
    Builder { name: name.to_string(), ..self }
  }

  pub fn description(self, description: impl ToString) -> Builder {
    Builder { description: description.to_string(), ..self }
  }

  pub fn unit(self, unit: impl ToString) -> Builder {
    Builder { unit: unit.to_string(), ..self }
  }

  pub fn typ(self, typ: Type) -> Builder {
    Builder { typ: Some(typ), ..self }
  }

  pub fn tags(self, tags: impl IntoIterator<Item = impl ToString>) -> Builder {
    Builder { tags: tags.into_iter().map(|s| s.to_string()).collect(), ..self }
  }
}
