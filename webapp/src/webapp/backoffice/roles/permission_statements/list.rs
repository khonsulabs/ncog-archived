use crate::webapp::{
    backoffice::{
        entity_list::{body::EntityRenderer, EntityList},
        roles::permission_statements::fields::PermissionStatementFields,
    },
    strings::LocalizableName,
    AppRoute, EditingId,
};
use shared::iam::PermissionStatement;
use std::{rc::Rc, sync::RwLock};
use yew::prelude::*;
use yew_router::prelude::*;

pub fn standard_head() -> Html {
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

pub fn standard_row() -> EntityRenderer<PermissionStatement> {
    row(standard_actions)
}

pub fn row<F: Fn(&PermissionStatement) -> Html + 'static>(
    actions: F,
) -> EntityRenderer<PermissionStatement> {
    EntityRenderer::new(move |permission_statement: &PermissionStatement| {
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
                    { actions(permission_statement) }
                </td>
            </tr>
        }
    })
}

fn standard_actions(permission_statement: &PermissionStatement) -> Html {
    html! {
        <RouterButton<AppRoute> route=AppRoute::BackOfficeRolePermissionStatementEdit(permission_statement.role_id.unwrap(), EditingId::Id(permission_statement.id.unwrap())) classes="button is-primary" >
            <strong>{ localize!("edit") }</strong>
        </RouterButton<AppRoute>>
    }
}

fn render_property<T: Into<Html>>(value: Option<T>, default_label: &'static str) -> Html {
    match value {
        Some(value) => value.into(),
        None => {
            html! {<span class="tag is-success is-medium">{ localize!(default_label)}</span>}
        }
    }
}

pub fn standard(entities: Option<Rc<RwLock<Vec<PermissionStatement>>>>) -> Html {
    html! {
        <EntityList<PermissionStatement>
            header=standard_head()
            row=standard_row()
            entities=entities
            />
    }
}
