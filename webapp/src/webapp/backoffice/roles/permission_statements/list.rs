use crate::webapp::{
    backoffice::{
        entity_list::ListableEntity,
        roles::permission_statements::fields::PermissionStatementFields,
    },
    strings::LocalizableName,
    AppRoute, EditingId,
};
use shared::iam::PermissionStatement;
use std::rc::Rc;
use yew::prelude::*;
use yew_router::prelude::*;

impl ListableEntity for PermissionStatement {
    type Entity = Self;

    fn table_head() -> Html {
        html! {
            <tr>
                <td>{ PermissionStatementFields::Id.localized_name() }</td>
                <td>{ PermissionStatementFields::Service.localized_name() }</td>
                <td>{ PermissionStatementFields::ResourceType.localized_name() }</td>
                <td>{ PermissionStatementFields::ResourceId.localized_name() }</td>
                <td>{ PermissionStatementFields::Action.localized_name() }</td>
                <td>{ PermissionStatementFields::Allow.localized_name() }</td>
                <td></td>
            </tr>
        }
    }

    fn render_entity(
        permission_statement: &Self::Entity,
        action_buttons: Option<Rc<Box<dyn Fn(&Self::Entity) -> Html>>>,
    ) -> Html {
        let allowed = if permission_statement.allow {
            html! { <span class="tag is-success is-medium">{ localize!("action-allowed")}</span> }
        } else {
            html! { <span class="tag is-danger is-medium">{ localize!("action-denied")}</span> }
        };

        html! {
            <tr>
                <td>{ permission_statement.id.unwrap() }</td>
                <td>{ render_property(permission_statement.service.as_ref(), "any-service") }</td>
                <td>{ render_property(permission_statement.resource_type.as_ref(), "any-resource-type") }</td>
                <td>{ render_property(permission_statement.resource_id.as_ref(), "any-resource-id") }</td>
                <td>{ render_property(permission_statement.action.as_ref(), "any-action") }</td>
                <td>{ allowed }</td>
                <td>
                    { Self::render_action_buttons(action_buttons, permission_statement) }
                </td>
            </tr>
        }
    }
}

fn render_property<T: Into<Html>>(value: Option<T>, default_label: &'static str) -> Html {
    match value {
        Some(value) => value.into(),
        None => {
            html! {<span class="tag is-primary is-medium">{ localize!(default_label)}</span>}
        }
    }
}

pub fn default_action_buttons() -> Option<Rc<Box<dyn Fn(&PermissionStatement) -> Html>>> {
    Some(Rc::new(Box::new(render_default_action_buttons)))
}

fn render_default_action_buttons(stmt: &PermissionStatement) -> Html {
    html! {
        <RouterButton<AppRoute> route=AppRoute::BackOfficeRolePermissionStatementEdit(EditingId::Id(stmt.id.unwrap())) classes="button is-primary" >
            <strong>{ localize!("edit") }</strong>
        </RouterButton<AppRoute>>
    }
}
