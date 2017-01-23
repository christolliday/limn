use input::{self, EventId};

use widget::{self, EventHandler, DrawableEventHandler, EventArgs};
use event::{self, EventAddress};
use widgets::primitives::{self, RectDrawable};
use widgets::text::{self, TextDrawable};
use widget::builder::WidgetBuilder;
use util::Dimensions;
use resources::Id;
use color::*;

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
    fn handle_event(&mut self, event_args: EventArgs) {
        let EventArgs { event, widget_id, event_queue, .. } = event_args;
        let event = event.data::<input::Event>();

        self.on = !self.on;
        let event = if self.on {
            event::InputEvent::new(event::BUTTON_ENABLED, event.clone())
        } else {
            event::InputEvent::new(event::BUTTON_DISABLED, event.clone())
        };
        event_queue.push(EventAddress::SubTree(widget_id), Box::new(event));
    }
}


pub struct ToggleButtonBuilder {
    pub widget: WidgetBuilder,
}
impl ToggleButtonBuilder {
    pub fn new() -> Self {
        let rect = RectDrawable { background: RED };
        let mut widget = WidgetBuilder::new();
        widget.set_drawable(primitives::draw_rect, Box::new(rect));
        widget.event_handlers.push(Box::new(ToggleEventHandler::new()));

        fn set_rect_on(state: &mut RectDrawable) {
            state.background = WHITE;
        };
        fn set_rect_off(state: &mut RectDrawable) {
            state.background = RED;
        };
        widget.event_handlers
            .push(Box::new(DrawableEventHandler::new(event::BUTTON_ENABLED, set_rect_on)));
        widget.event_handlers
            .push(Box::new(DrawableEventHandler::new(event::BUTTON_DISABLED, set_rect_off)));
        widget.layout.dimensions(Dimensions {
            width: 100.0,
            height: 50.0,
        });

        ToggleButtonBuilder { widget: widget }
    }
    pub fn set_text(&mut self,
                    on_text: &'static str,
                    off_text: &'static str,
                    font_id: Id) {

        let set_text_on = move |state: &mut TextDrawable| {
            state.text = on_text.to_owned();
        };
        let set_text_off = move |state: &mut TextDrawable| {
            state.text = off_text.to_owned();
        };
        let button_text_drawable = TextDrawable {
            text: off_text.to_owned(),
            font_id: font_id,
            font_size: 20.0,
            text_color: BLACK,
            background_color: TRANSPARENT,
        };
        let button_text_dims = button_text_drawable.measure_dims_no_wrap();
        let mut button_text_widget = WidgetBuilder::new();
        button_text_widget.set_drawable(text::draw_text, Box::new(button_text_drawable));
        button_text_widget.event_handlers
            .push(Box::new(DrawableEventHandler::new(event::BUTTON_ENABLED, set_text_on)));
        button_text_widget.event_handlers
            .push(Box::new(DrawableEventHandler::new(event::BUTTON_DISABLED, set_text_off)));
        button_text_widget.layout.dimensions(button_text_dims);
        button_text_widget.layout.center(&self.widget.layout);

        self.widget.add_child(Box::new(button_text_widget));
    }
    pub fn builder(self) -> WidgetBuilder {
        self.widget
    }
}

pub struct PushButtonBuilder {
    pub widget: WidgetBuilder,
}
impl PushButtonBuilder {
    pub fn new() -> Self {
        let rect = RectDrawable { background: RED };
        let mut widget = WidgetBuilder::new();
        widget.set_drawable(primitives::draw_rect, Box::new(rect));

        widget.layout.dimensions(Dimensions {
            width: 100.0,
            height: 50.0,
        });

        PushButtonBuilder { widget: widget }
    }
    pub fn set_text(&mut self, text: &'static str, font_id: Id) {
        let button_text_drawable = TextDrawable {
            text: text.to_owned(),
            font_id: font_id,
            font_size: 20.0,
            text_color: BLACK,
            background_color: TRANSPARENT,
        };
        let button_text_dims = button_text_drawable.measure_dims_no_wrap();
        let mut button_text_widget = WidgetBuilder::new();
        button_text_widget.set_drawable(text::draw_text, Box::new(button_text_drawable));
        button_text_widget.layout.dimensions(button_text_dims);
        button_text_widget.layout.center(&self.widget.layout);

        self.widget.add_child(Box::new(button_text_widget));
    }
    pub fn builder(self) -> WidgetBuilder {
        self.widget
    }
}
