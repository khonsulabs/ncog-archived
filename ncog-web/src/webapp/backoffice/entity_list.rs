use std::{rc::Rc, sync::RwLock};
use yew::prelude::*;

pub mod body;

pub struct EntityList<T>
where
    T: Clone + 'static,
{
    props: Props<T>,
}

#[derive(Clone, Properties)]
pub struct Props<T>
where
    T: Clone + 'static,
{
    pub header: Html,
    pub entities: Option<Rc<RwLock<Vec<T>>>>,
    pub row: body::EntityRenderer<T>,
}

impl<T> Component for EntityList<T>
where
    T: Clone + 'static,
{
    type Message = ();
    type Properties = Props<T>;
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
                    { self.props.header.clone() }
                    <body::EntityListBody<T> entities=entities.clone() render=self.props.row.clone() />
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
