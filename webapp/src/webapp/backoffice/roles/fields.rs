
use crate::webapp::strings::Namable;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum RoleFields {
    Id,
    Name,
    CreatedAt,
}

impl Namable for RoleFields {
    fn name(&self) -> &'static str {
        match self {
            Self::Id => "role-fields-id",
            Self::Name => "role-fields-name",
            Self::CreatedAt => "role-fields-created-at",
        }
    }
}
