use yew::prelude::*;
use yew::virtual_dom::VTag;

pub struct Title {
    props: Props,
}

#[derive(Clone, Properties)]
pub struct Props {
    #[prop_or_default]
    pub size: Option<i8>,
    #[prop_or_default]
    pub subtitle: bool,
    #[prop_or_default]
    pub children: Children,
}

impl Component for Title {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        self.props = props;
        true
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let size = self.effective_size();
        let css_class = format!("{} is-{}", self.primary_class(), size);

        let mut heading = VTag::new(format!("h{}", size));
        heading.add_attribute("class", &css_class);
        heading.add_child(self.props.children.iter().collect::<Html>());
        heading.into()
    }
}

impl Title {
    fn primary_class(&self) -> &'static str {
        if self.props.subtitle {
            "subtitle"
        } else {
            "title"
        }
    }

    fn effective_size(&self) -> i8 {
        match self.props.size {
            Some(size) => match size {
                1..=6 => size,
                _ => Self::default_size(self.props.subtitle),
            },
            None => Self::default_size(self.props.subtitle),
        }
    }

    fn default_size(subtitle: bool) -> i8 {
        // Defaults chosen from bulma's defaults https://bulma.io/documentation/elements/title/
        if subtitle {
            5
        } else {
            3
        }
    }
}
