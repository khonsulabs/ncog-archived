use crate::webapp::strings::Namable;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum PermissionStatementFields {
    Id,
    Service,
    ResourceType,
    ResourceId,
    Action,
    Allow,
}

impl Namable for PermissionStatementFields {
    fn name(&self) -> &'static str {
        use PermissionStatementFields::*;
        match self {
            Id => "permission-statements-id",
            Service => "permission-statements-service",
            ResourceType => "permission-statements-resource-type",
            ResourceId => "permission-statements-resource-id",
            Action => "permission-statements-action",
            Allow => "permission-statements-allow",
        }
    }
}
