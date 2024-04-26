use crate::entity::journal::Root;
use crate::entity::{normalize_description, normalize_name, normalize_tags, normalize_unit};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct Builder {
  id: Option<Uuid>,
  name: String,
  description: String,
  unit: String,
  tags: HashSet<String>,
}

impl From<Root> for Builder {
  fn from(value: Root) -> Self {
    Builder {
      id: Some(value.id),
      name: value.name,
      description: value.description,
      unit: value.unit,
      tags: value.tags,
    }
  }
}

impl Builder {
  pub fn build(self) -> crate::Result<Root> {
    let name = normalize_name(crate::entity::journal::TYPE, self.name)?;
    let description = normalize_description(crate::entity::journal::TYPE, self.description)?;
    let unit = normalize_unit(crate::entity::journal::TYPE, self.unit)?;
    let tags = normalize_tags(crate::entity::journal::TYPE, self.tags)?;
    Ok(Root { id: self.id.unwrap_or_else(Uuid::new_v4), name, description, unit, tags })
  }

  pub fn id(self, id: Uuid) -> Builder {
    Builder { id: Some(id), ..self }
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

  pub fn tags(self, tags: impl IntoIterator<Item = impl ToString>) -> Builder {
    Builder { tags: tags.into_iter().map(|s| s.to_string()).collect(), ..self }
  }
}
