use crate::webapp::{
    backoffice::{
        entity_list::{body::EntityRenderer, EntityList},
        roles::fields::RoleFields,
    },
    strings::Namable,
    AppRoute, EditingId,
};
use ncog_shared::iam::RoleSummary;
use std::{rc::Rc, sync::RwLock};
use yew::prelude::*;
use yew_router::prelude::*;

pub fn standard_head() -> Html {
    html! {
        <tr>
            <td>{ RoleFields::Id.localized_name() }</td>
            <td>{ RoleFields::Name.localized_name() }</td>
            <td></td>
        </tr>
    }
}

pub fn standard(entities: Option<Rc<RwLock<Vec<RoleSummary>>>>) -> Html {
    html! {
        <EntityList<RoleSummary>
            row=standard_row()
            header=standard_head()
            entities=entities
            />
    }
}

pub fn standard_row() -> EntityRenderer<RoleSummary> {
    row(standard_actions)
}

pub fn row<F: Fn(&RoleSummary) -> Html + 'static>(actions: F) -> EntityRenderer<RoleSummary> {
    EntityRenderer::new(move |role: &RoleSummary| {
        html! {
            <tr>
                <td>{ role.id.unwrap() }</td>
                <td>{ &role.name }</td>
                <td>
                    { actions(role) }
                </td>
            </tr>
        }
    })
}

fn standard_actions(role: &RoleSummary) -> Html {
    html! {
        <RouterButton<AppRoute> route=AppRoute::BackOfficeRoleEdit(EditingId::Id(role.id.unwrap())) classes="button is-primary" >
            <strong>{ localize!("edit") }</strong>
        </RouterButton<AppRoute>>
    }
}
