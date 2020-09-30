use crate::webapp::strings::Namable;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum UserFields {
    Id,
    Screenname,
    CreatedAt,
    AssignedRoles,
}

impl Namable for UserFields {
    fn name(&self) -> &'static str {
        match self {
            Self::Id => "user-fields-id",
            Self::Screenname => "user-fields-screenname",
            Self::CreatedAt => "user-fields-created-at",
            Self::AssignedRoles => "user-fields-assigned-roles",
        }
    }
}
