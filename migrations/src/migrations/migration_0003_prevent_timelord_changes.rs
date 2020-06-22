use super::TIMELORD_ROLE_ID;
use sqlx_simple_migrator::Migration;

pub fn migration() -> Migration {
    Migration::new(std::file!())
        .with_up(&format!(
            "INSERT INTO role_permission_statements (service, resource_type, resource_id, action, allow) values ('iam', 'roles', {}, 'update', false)",
            TIMELORD_ROLE_ID
        ))
        .with_down(&format!(
            "DELETE FROM role_permission_statements WHERE service = 'iam' and resource_type = 'roles' and resource_id = {} and action = 'update' and role_id is null",
            TIMELORD_ROLE_ID
        ))
        .with_up(&format!(
            "INSERT INTO role_permission_statements (service, resource_type, resource_id, action, allow) values ('iam', 'roles', {}, 'delete', false)",
            TIMELORD_ROLE_ID
        ))
        .with_down(&format!(
            "DELETE FROM role_permission_statements WHERE service = 'iam' and resource_type = 'roles' and resource_id = {} and action = 'delete' and role_id is null",
            TIMELORD_ROLE_ID
        ))
        .with_up("ALTER TABLE role_permission_statements DROP CONSTRAINT role_permission_statements_role_id_fkey")
        .with_up("ALTER TABLE role_permission_statements ADD CONSTRAINT role_permission_statements_role_id_fkey FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE")
        .with_down("ALTER TABLE role_permission_statements ADD CONSTRAINT role_permission_statements_role_id_fkey FOREIGN KEY (role_id) REFERENCES roles(id)")
        .with_down("ALTER TABLE role_permission_statements DROP CONSTRAINT role_permission_statements_role_id_fkey")
}
