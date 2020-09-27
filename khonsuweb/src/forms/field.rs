use super::label::Label;
use std::{collections::HashMap, rc::Rc};
use yew::prelude::*;

pub struct Field<T>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
{
    props: Props<T>,
}

#[derive(Clone, Properties)]
pub struct Props<T>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
{
    pub field: T,
    pub errors: Option<Rc<HashMap<T, Vec<Rc<Html>>>>>,
    #[prop_or_default]
    pub label: String,
    #[prop_or_default]
    pub help: String,
    #[prop_or_default]
    pub children: Children,
}

impl<T> Component for Field<T>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
{
    type Message = ();
    type Properties = Props<T>;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let label = if !self.props.label.is_empty() {
            html! {<Label text=self.props.label.clone()/>}
        } else {
            html! {}
        };
        let help = if !self.props.help.is_empty() {
            html! {<p class="help">{ &self.props.help }</p>}
        } else {
            html! {}
        };
        let errors = self.props.errors.as_ref().map(|errors| {
            errors.get(&self.props.field).map(|errors| {
                errors
                    .iter()
                    .map(|e| html! {<p class="help is-danger">{e.as_ref().clone()}</p>})
                    .collect::<Html>()
            })
        });

        let error_message = match errors {
            Some(errors) => match errors {
                Some(field_errors) => field_errors,
                None => Html::default(),
            },
            None => Html::default(),
        };
        html! {
            <div class="field">
                { label }
                { self.props.children.clone() }
                { error_message }
                { help }
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}
