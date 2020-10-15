use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Claim {
    service: String,
    resource_type: Option<String>,
    resource_id: Option<i64>,
    action: String,
}

impl Claim {
    pub fn new<S: Into<String>>(
        service: S,
        resource_type: Option<S>,
        resource_id: Option<i64>,
        action: S,
    ) -> Self {
        Self {
            service: service.into(),
            resource_type: resource_type.map(|r| r.into()),
            resource_id,
            action: action.into(),
        }
    }
}

pub struct Statement {
    pub role_id: Option<i64>,
    pub service: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<i64>,

    pub action: Option<String>,

    pub allow: bool,
}

impl Statement {
    #[cfg(test)]
    fn new<S: Into<String>>(
        role_id: Option<i64>,
        service: Option<S>,
        resource_type: Option<S>,
        resource_id: Option<i64>,

        action: Option<S>,

        allow: bool,
    ) -> Self {
        Self {
            role_id,
            service: service.map(|s| s.into()),
            resource_type: resource_type.map(|s| s.into()),
            resource_id,
            action: action.map(|s| s.into()),
            allow,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PermissionSet {
    service_permissions: HashMap<Option<String>, ServicePermission>,
    pub role_ids: HashSet<i64>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JsonPermissionSet {
    service_permissions: HashMap<String, JsonResourceTypePermissions>,
    role_ids: Vec<i64>,
}

type JsonActionPermissions = HashMap<String, bool>;
type JsonResourcePermissions = HashMap<String, JsonActionPermissions>;
type JsonResourceTypePermissions = HashMap<String, JsonResourcePermissions>;

impl From<JsonPermissionSet> for PermissionSet {
    fn from(input: JsonPermissionSet) -> Self {
        let mut service_permissions = HashMap::new();

        for (service, in_service_permission) in input.service_permissions {
            service_permissions.insert(
                if service.is_empty() {
                    None
                } else {
                    Some(service)
                },
                ServicePermission::from(in_service_permission),
            );
        }

        Self {
            service_permissions,
            role_ids: input.role_ids.into_iter().collect(),
        }
    }
}

impl From<JsonResourceTypePermissions> for ServicePermission {
    fn from(input: JsonResourceTypePermissions) -> Self {
        let mut resource_type_permissions = HashMap::new();

        for (resource_type, in_service_permission) in input {
            resource_type_permissions.insert(
                if resource_type.is_empty() {
                    None
                } else {
                    Some(resource_type)
                },
                ResourceTypePermission::from(in_service_permission),
            );
        }

        Self {
            resource_type_permissions,
        }
    }
}

impl From<JsonResourcePermissions> for ResourceTypePermission {
    fn from(input: JsonResourcePermissions) -> Self {
        let mut resource_permissions = HashMap::new();

        for (resource, in_service_permission) in input {
            resource_permissions.insert(
                if resource.is_empty() {
                    None
                } else {
                    Some(resource.parse().unwrap_or_default())
                },
                ResourcePermission::from(in_service_permission),
            );
        }

        Self {
            resource_permissions,
        }
    }
}

impl From<JsonActionPermissions> for ResourcePermission {
    fn from(input: JsonActionPermissions) -> Self {
        let mut action_permissions = HashMap::new();

        for (service, in_service_permission) in input {
            action_permissions.insert(
                if service.is_empty() {
                    None
                } else {
                    Some(service)
                },
                in_service_permission,
            );
        }

        Self { action_permissions }
    }
}

impl From<PermissionSet> for JsonPermissionSet {
    fn from(permission_set: PermissionSet) -> Self {
        let mut service_permissions = HashMap::new();
        for (service, service_permission) in permission_set.service_permissions {
            let mut resource_type_permissions = HashMap::new();

            for (resource_type, resource_type_permission) in
                service_permission.resource_type_permissions.iter()
            {
                let mut resource_permissions = HashMap::new();

                for (resource, resource_permission) in
                    resource_type_permission.resource_permissions.iter()
                {
                    let mut action_permissions = HashMap::new();
                    for (action, allowed) in resource_permission.action_permissions.iter() {
                        action_permissions.insert(action.clone().unwrap_or_default(), *allowed);
                    }

                    resource_permissions.insert(
                        resource.map(|id| id.to_string()).unwrap_or_default(),
                        action_permissions,
                    );
                }

                resource_type_permissions.insert(
                    resource_type.clone().unwrap_or_default(),
                    resource_permissions,
                );
            }

            service_permissions.insert(
                service.clone().unwrap_or_default(),
                resource_type_permissions,
            );
        }

        JsonPermissionSet {
            service_permissions,
            role_ids: permission_set.role_ids.into_iter().collect(),
        }
    }
}

impl From<Vec<Statement>> for PermissionSet {
    fn from(statements: Vec<Statement>) -> Self {
        let mut set = PermissionSet::default();

        for statement in statements {
            if let Some(role_id) = statement.role_id {
                set.role_ids.insert(role_id);
            }

            set.service_permissions
                .entry(statement.service.clone())
                .and_modify(|service_permission| service_permission.apply(&statement))
                .or_insert_with(|| ServicePermission::from_statement(&statement));
        }

        set
    }
}

impl PermissionSet {
    pub fn allowed(&self, claim: &Claim) -> bool {
        if let Some(service_permission) = self.service_permissions.get(&Some(claim.service.clone()))
        {
            if let Some(allowed) = service_permission.allowed(claim) {
                return allowed;
            }
        }

        if let Some(generic_permission) = self.service_permissions.get(&None) {
            if let Some(allowed) = generic_permission.allowed(claim) {
                return allowed;
            }
        }

        false
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ServicePermission {
    resource_type_permissions: HashMap<Option<String>, ResourceTypePermission>,
}

impl ServicePermission {
    fn from_statement(statement: &Statement) -> Self {
        let mut perm = ServicePermission::default();
        perm.apply(statement);
        perm
    }

    pub fn allowed(&self, claim: &Claim) -> Option<bool> {
        if let Some(claimed_type) = &claim.resource_type {
            if let Some(resource_type_permission) = self
                .resource_type_permissions
                .get(&Some(claimed_type.clone()))
            {
                return resource_type_permission.allowed(claim);
            }
        }

        if let Some(generic_permission) = self.resource_type_permissions.get(&None) {
            return generic_permission.allowed(claim);
        }

        None
    }

    fn apply(&mut self, statement: &Statement) {
        self.resource_type_permissions
            .entry(statement.resource_type.clone())
            .and_modify(|rtp| rtp.apply(statement))
            .or_insert_with(|| ResourceTypePermission::from_statement(statement));
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ResourceTypePermission {
    resource_permissions: HashMap<Option<i64>, ResourcePermission>,
}

impl ResourceTypePermission {
    fn from_statement(statement: &Statement) -> Self {
        let mut perm = ResourceTypePermission::default();
        perm.apply(&statement);
        perm
    }
    pub fn allowed(&self, claim: &Claim) -> Option<bool> {
        if let Some(claimed_id) = &claim.resource_id {
            if let Some(resource_permission) = self.resource_permissions.get(&Some(*claimed_id)) {
                if let Some(allowed) = resource_permission.allowed(claim) {
                    return Some(allowed);
                }
            }
        }

        if let Some(generic_permission) = self.resource_permissions.get(&None) {
            if let Some(allowed) = generic_permission.allowed(claim) {
                return Some(allowed);
            }
        }

        None
    }

    fn apply(&mut self, statement: &Statement) {
        self.resource_permissions
            .entry(statement.resource_id)
            .and_modify(|rtp| rtp.apply(statement))
            .or_insert_with(|| ResourcePermission::from_statement(statement));
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ResourcePermission {
    action_permissions: HashMap<Option<String>, bool>,
}

impl ResourcePermission {
    fn from_statement(statement: &Statement) -> Self {
        let mut perm = ResourcePermission::default();
        perm.apply(&statement);
        perm
    }

    pub fn allowed(&self, claim: &Claim) -> Option<bool> {
        if let Some(action_permission) = self.action_permissions.get(&Some(claim.action.clone())) {
            return Some(*action_permission);
        }

        if let Some(generic_permission) = self.action_permissions.get(&None) {
            return Some(*generic_permission);
        }

        None
    }

    fn apply(&mut self, statement: &Statement) {
        self.action_permissions
            .entry(statement.action.clone())
            .and_modify(|allowed| *allowed = statement.allow)
            .or_insert_with(|| statement.allow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_permissions() -> PermissionSet {
        PermissionSet::from(vec![
            // Allow read for everyhing
            Statement::new(None, None, None, None, Some("read"), true),
            // Allow everything for id 1
            Statement::new(None, Option::<String>::None, None, Some(1i64), None, true),
            // Allow everything for resource type 'always-type'
            Statement::new(None, None, Some("always-type"), None, None, true),
            // Allow everything for resource type 'always-type'
            Statement::new(None, Some("always-service"), None, None, None, true),
            // Deny reading for a specific id
            Statement::new(None, None, None, Some(13i64), Some("read"), false),
            // Deny reading for a specific type
            Statement::new(None, None, Some("deny-type"), None, Some("read"), false),
            // Deny reading for a specific service
            Statement::new(None, Some("deny-service"), None, None, Some("read"), false),
        ])
    }

    #[test]
    fn default_deny() {
        let set = test_permissions();
        assert!(!set.allowed(&Claim::new(
            "nonexistant-service",
            Some("nonexistant-type"),
            Some(i64::MAX),
            "nonexistant-action"
        )));
    }

    #[test]
    fn deny_by_id() {
        let set = test_permissions();
        assert!(!set.allowed(&Claim::new(
            "nonexistant-service",
            Some("nonexistant-type"),
            Some(13i64),
            "read"
        )));
    }

    #[test]
    fn deny_by_type() {
        let set = test_permissions();
        assert!(!set.allowed(&Claim::new(
            "nonexistant-service",
            Some("deny-type"),
            Some(i64::MAX),
            "read"
        )));
    }

    #[test]
    fn deny_by_service() {
        let set = test_permissions();
        assert!(!set.allowed(&Claim::new(
            "deny-service",
            Some("nonexistant-type"),
            Some(i64::MAX),
            "read"
        )));
    }

    #[test]
    fn stranded_action_leaf_test() {
        let set = test_permissions();
        assert!(set.allowed(&Claim::new(
            "nonexistant-service",
            Some("nonexistant-type"),
            Some(i64::MAX),
            "read"
        )));
    }

    #[test]
    fn stranded_id_leaf_test() {
        let set = test_permissions();
        assert!(set.allowed(&Claim::new(
            "nonexistant-service",
            Some("nonexistant-type"),
            Some(1),
            "read"
        )));
    }

    #[test]
    fn stranded_resource_type_leaf_test() {
        let set = test_permissions();
        assert!(set.allowed(&Claim::new(
            "nonexistant-service",
            Some("always-type"),
            Some(i64::MAX),
            "nonexistant-action"
        )));
    }

    #[test]
    fn stranded_service_leaf_test() {
        let set = test_permissions();
        assert!(set.allowed(&Claim::new(
            "always-service",
            Some("nonexistant-type"),
            Some(i64::MAX),
            "nonexistant-action"
        )));
    }

    #[test]
    fn json_conversion() {
        let set = test_permissions();
        let json = JsonPermissionSet::from(set.clone());
        let back = PermissionSet::from(json);
        assert_eq!(set, back);
    }
}
