use std::rc::Rc;
use yew::prelude::*;

pub type RenderFunction<T> = Box<dyn Fn(&T) -> Html>;

pub trait ListableEntity {
    type Entity: Clone;

    fn table_head() -> Html;
    fn render_entity(
        entity: &Self::Entity,
        action_buttons: Option<Rc<RenderFunction<Self::Entity>>>,
    ) -> Html;

    fn render_action_buttons(
        action_buttons: Option<Rc<RenderFunction<Self::Entity>>>,
        entity: &Self::Entity,
    ) -> Html {
        action_buttons
            .map(|buttons| buttons(entity))
            .unwrap_or_default()
    }
}

pub trait ActionableEntity: ListableEntity {
    type ButtonProvider: ActionButtonProvider<Self::Entity>;
}

pub trait ActionButtonProvider<T> {
    fn action_buttons_for(entity: &T) -> Html;
}

pub struct EntityList<T>
where
    T: ListableEntity,
{
    props: Props<T::Entity>,
}

#[derive(Clone, Properties)]
pub struct Props<T: Clone> {
    pub entities: Option<Vec<T>>,
    #[prop_or_default]
    pub action_buttons: Option<Rc<RenderFunction<T>>>,
}

impl<T> Component for EntityList<T>
where
    T: ListableEntity + 'static,
{
    type Message = ();
    type Properties = Props<T::Entity>;
    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        match &self.props.entities {
            Some(entities) => html!(
                <table class="table is-hoverable is-striped">
                    <thead>
                        {T::table_head()}
                    </thead>
                    <tbody>
                        { entities.iter().map(|u| T::render_entity(u, self.props.action_buttons.clone())).collect::<Html>() }
                    </tbody>
                </table>
            ),
            None => {
                html! {
                    <progress class="progress is-primary" max="100"/>
                }
            }
        }
    }
}
