use super::EventHandler;
use super::super::event;
use event::Event;
use input::{EventId};
use input;
use std::any::Any;
use super::primitives::{RectDrawable, EllipseDrawable};
use super::text::TextDrawable;
use super::layout::WidgetLayout;
use widget::DrawableEventHandler;
use widget::builder::WidgetBuilder;
use widget::EventArgs;
use widget;
use graphics::types::Color;
use util::{Scalar, Dimensions};
use resources::Id;
use ui::Resources;

use cassowary::Solver;

pub struct ToggleEventHandler {
    on: bool,
}
impl ToggleEventHandler {
    pub fn new() -> ToggleEventHandler {
        ToggleEventHandler { on: false }
    }
}
impl EventHandler for ToggleEventHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_PRESS
    }
    fn handle_event(&mut self, event_args: EventArgs) -> Option<Box<Event>> {
        let EventArgs { event, .. } = event_args;
        let event: &input::Event = event.event_data().downcast_ref().unwrap();
        
        self.on = !self.on;
        let event = if self.on {
            event::InputEvent::new(event::BUTTON_ENABLED, event.clone())
        } else {
            event::InputEvent::new(event::BUTTON_DISABLED, event.clone())
        };
        Some(Box::new(event))
    }
}


pub struct ButtonBuilder<'a> {
    pub widget: WidgetBuilder,
    resources: &'a Resources,
}
impl<'a> ButtonBuilder<'a> {
    pub fn new(resources: &'a Resources) -> Self {
        let rect = RectDrawable { background: [1.0, 0.0, 0.0, 1.0] };
        let mut widget = WidgetBuilder::new();
        widget.set_drawable(widget::primitives::draw_rect, Box::new(rect));
        widget.event_handlers.push(Box::new(ToggleEventHandler::new()));

        fn set_rect_on(state: &mut RectDrawable) {
            state.background = [4.0, 1.0, 1.0, 1.0];
        };
        fn set_rect_off(state: &mut RectDrawable) {
            state.background = [1.0, 0.0, 0.0, 1.0];
        };
        widget.event_handlers.push(Box::new(DrawableEventHandler::new(event::BUTTON_ENABLED, set_rect_on)));
        widget.event_handlers.push(Box::new(DrawableEventHandler::new(event::BUTTON_DISABLED, set_rect_off)));
        widget.debug_color([1.0, 1.0, 0.0, 1.0]);
        widget.layout.dimensions(Dimensions { width: 100.0, height: 50.0 });
        
        ButtonBuilder { widget: widget, resources: resources }
    }
    pub fn set_text(&mut self, on_text: &'static str, off_text: &'static str, font_id: Id, font_size: Scalar, text_color: Color) {

        let set_text_on = move |state: &mut TextDrawable| {
            state.text = on_text.to_owned();
        };
        let set_text_off = move |state: &mut TextDrawable| {
            state.text = off_text.to_owned();
        };
        let button_text_drawable = TextDrawable { text: off_text.to_owned(), font_id: font_id, font_size: 20.0, text_color: [0.0,0.0,0.0,1.0], background_color: [0.0, 0.0, 0.0, 0.0] };
        let button_text_dims = button_text_drawable.measure_dims_no_wrap(self.resources);
        let mut button_text_widget = WidgetBuilder::new();
        button_text_widget.set_drawable(widget::text::draw_text, Box::new(button_text_drawable));
        button_text_widget.event_handlers.push(Box::new(DrawableEventHandler::new(event::BUTTON_ENABLED, set_text_on)));
        button_text_widget.event_handlers.push(Box::new(DrawableEventHandler::new(event::BUTTON_DISABLED, set_text_off)));
        button_text_widget.layout.dimensions(button_text_dims);
        button_text_widget.layout.center(&self.widget.layout);

        self.widget.add_child(Box::new(button_text_widget));
    }
    pub fn builder(self) -> WidgetBuilder {
        self.widget
    }
}