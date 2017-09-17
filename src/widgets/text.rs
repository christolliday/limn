use cassowary::Constraint;

use widget::WidgetBuilder;
use widget::style::StyleUpdated;
use drawable::text::{TextDrawable, TextStyleable};
use event::{WidgetEventHandler, WidgetEventArgs};
use layout::constraint::*;

pub struct TextBuilder;

impl TextBuilder {
    pub fn new(text: &str) -> WidgetBuilder {
        let text_drawable = TextDrawable::new(text);
        let mut widget = WidgetBuilder::new(text);
        widget.set_drawable(text_drawable);
        widget.add_handler(TextUpdatedHandler { size_constraints: Vec::new() });
        widget
    }
    pub fn new_with_style(style: Vec<TextStyleable>) -> WidgetBuilder {
        let text_drawable = TextDrawable::default();
        let mut widget = WidgetBuilder::new("text");
        widget.set_drawable_with_style(text_drawable, style);
        widget.add_handler(TextUpdatedHandler::default());
        widget
    }
}

#[derive(Default)]
struct TextUpdatedHandler {
    size_constraints: Vec<Constraint>,
}
impl WidgetEventHandler<StyleUpdated> for TextUpdatedHandler {
    fn handle(&mut self, _: &StyleUpdated, mut args: WidgetEventArgs) {
        args.widget.update_layout(|layout| {
            for constraint in self.size_constraints.drain(..) {
                layout.remove_constraint(constraint);
            }
        });
        let text_size = {
            let drawable = args.widget.drawable();
            let text_drawable = drawable.downcast_ref::<TextDrawable>().unwrap();
            text_drawable.measure()
        };
        let size_constraints = size(text_size).build(&args.widget.layout_vars());
        args.widget.update_layout(|layout| {
            layout.add(size_constraints.clone());
        });
        self.size_constraints = size_constraints;
    }
}
