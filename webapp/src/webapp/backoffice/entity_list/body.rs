use std::{rc::Rc, sync::RwLock};
use yew::prelude::*;

pub trait RenderFunction<T>: Fn(&T) -> Html {}
impl<F, T> RenderFunction<T> for F
where
    F: Fn(&T) -> Html,
    T: Clone + 'static,
{
}

#[derive(Clone)]
pub struct EntityRenderer<T>
where
    T: Clone + 'static,
{
    function: Rc<dyn RenderFunction<T>>,
}

impl<T> EntityRenderer<T>
where
    T: Clone + 'static,
{
    pub fn new<F: RenderFunction<T> + 'static>(function: F) -> Self {
        Self {
            function: Rc::new(function),
        }
    }
}

pub struct EntityListBody<T>
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
    pub entities: Option<Rc<RwLock<Vec<T>>>>,
    pub render: EntityRenderer<T>,
}

impl<T> Component for EntityListBody<T>
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
            Some(entities) => {
                let entities = entities.read().unwrap();
                html!(
                    <tbody>
                        { entities.iter().map(|u| self.props.render.function.as_ref()(u)).collect::<Html>() }
                    </tbody>
                )
            }
            None => {
                html! {
                    <progress class="progress is-primary" max="100"/>
                }
            }
        }
    }
}
