use crate::webapp::{
    backoffice::{
        entity_list::{ListableEntity, RenderFunction},
        roles::fields::RoleFields,
    },
    strings::LocalizableName,
    AppRoute, EditingId,
};
use shared::iam::RoleSummary;
use std::rc::Rc;
use yew::prelude::*;
use yew_router::prelude::*;

impl ListableEntity for RoleSummary {
    type Entity = RoleSummary;

    fn table_head() -> Html {
        html! {
            <tr>
                <td>{ RoleFields::Id.localized_name() }</td>
                <td>{ RoleFields::Name.localized_name() }</td>
                <td></td>
            </tr>
        }
    }

    fn render_entity(
        role: &Self::Entity,
        action_buttons: Option<Rc<RenderFunction<Self::Entity>>>,
    ) -> Html {
        html! {
            <tr>
                <td>{ role.id.unwrap() }</td>
                <td>{ &role.name }</td>
                <td>
                    { Self::render_action_buttons(action_buttons, role) }
                </td>
            </tr>
        }
    }
}

pub fn default_action_buttons() -> Option<Rc<RenderFunction<RoleSummary>>> {
    Some(Rc::new(Box::new(render_default_action_buttons)))
}

fn render_default_action_buttons(role: &RoleSummary) -> Html {
    html! {
        <RouterButton<AppRoute> route=AppRoute::BackOfficeRoleEdit(EditingId::Id(role.id.unwrap())) classes="button is-primary" >
            <strong>{ localize!("edit") }</strong>
        </RouterButton<AppRoute>>
    }
}
