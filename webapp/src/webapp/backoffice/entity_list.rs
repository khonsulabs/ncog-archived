use yew::prelude::*;

pub trait ListableEntity {
    type Entity: Clone;

    fn table_head() -> Html;
    fn render_entity(entity: &Self::Entity) -> Html;
}

pub struct EntityList<T>
where
    T: ListableEntity,
{
    props: Props<T::Entity>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props<T: Clone> {
    pub entities: Option<Vec<T>>,
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
                        { entities.iter().map(|u| T::render_entity(u)).collect::<Html>() }
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
