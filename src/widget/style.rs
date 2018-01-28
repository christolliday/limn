use std::any::TypeId;
use std::fmt;
use std::fmt::Debug;

use linked_hash_map::LinkedHashMap;
use style::DrawMergeStyle;
use style::ComponentStyle;
use style::Component;
use widget::property::PropSet;
use widget::draw::Draw;
use resources::resources;

#[derive(Default)]
pub struct DrawState {
    pub style_spec: Option<StyleSpec>,
    pub style: Option<Box<DrawMergeStyle>>,
    pub style_selector: LinkedHashMap<PropSet, Box<DrawMergeStyle>>,

    pub style_class: Option<String>,
    pub state: Option<Box<Draw>>,

    style_updated: bool,
}

impl Debug for DrawState {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

pub struct StyleSpec {
    get_style_fn: Box<Fn(&Option<Box<DrawMergeStyle>>, &LinkedHashMap<PropSet, Box<DrawMergeStyle>>, Option<String>, PropSet) -> Box<Draw>>,
}

impl StyleSpec {
    fn new<D: Draw + Component + 'static, T: ComponentStyle<Component = D> + Debug + Send + 'static>(_: &T) -> Self {
        let get_style_fn = |style: &_, selector: &_, class, props| {
            let res = resources();
            res.theme.get_style(style, selector, TypeId::of::<T>(), class, props)
        };
        StyleSpec {
            get_style_fn: Box::new(get_style_fn),
        }
    }
}
impl DrawState {

    pub fn style_updated(&mut self) {
        self.style_updated = true;
    }

    pub fn update(&mut self, props: PropSet) {
        if let Some(style_spec) = self.style_spec.as_ref() {
            let draw_state = (style_spec.get_style_fn)(&self.style, &self.style_selector, self.style_class.clone(), props);
            self.state = Some(draw_state);
            self.style_updated = false;
        }
    }

    pub fn needs_update(&self) -> bool {
        self.style_updated
    }
    pub fn set_draw_state<T: Draw + Component + 'static>(&mut self, draw_state: T) -> &mut Self {
        self.state = Some(Box::new(draw_state));
        self.style_updated();
        self
    }
    pub fn set_draw_style<D: Draw + Component + 'static, T: ComponentStyle<Component = D> + Debug + Send + 'static>(&mut self, style: T) {
        self.style_spec = Some(StyleSpec::new(&style));
        self.style = Some(Box::new(style));
        self.style_updated();
    }


    pub fn set_draw_style_prop<D: Draw + Component + 'static, T: ComponentStyle<Component = D> + Debug + Send + 'static>(&mut self, props: PropSet, style: T) {
        self.style_selector.insert(props, Box::new(style));
        self.style_updated();
    }

    pub fn set_style_class<D: Draw + Component + 'static, T: ComponentStyle<Component = D> + Debug + Send + 'static>(&mut self, style_class: &str, style: T) {
        self.style_spec = Some(StyleSpec::new(&style));
        self.style_class = Some(style_class.to_owned());
    }
}
