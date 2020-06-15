use crate::{
    localize,
    webapp::{
        backoffice::{entity_list::ListableEntity, roles::fields::RoleFields},
        strings::LocalizableName,
        AppRoute,
    },
};
use shared::iam::RoleSummary;
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

    fn render_entity(role: &Self::Entity) -> Html {
        html! {
            <tr>
                <td>{ role.id.unwrap() }</td>
                <td>{ &role.name }</td>
                <td>
                    <RouterButton<AppRoute> route=AppRoute::BackOfficeRoleEdit(role.id.unwrap()) classes="button is-primary" >
                        <strong>{ localize!("edit") }</strong>
                    </RouterButton<AppRoute>>
                </td>
            </tr>
        }
    }
}
