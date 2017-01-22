#[macro_use]
extern crate limn;
extern crate backend;
extern crate cassowary;
extern crate graphics;
extern crate input;
extern crate window;
extern crate find_folder;

#[macro_use]
extern crate matches;

use limn::ui::*;
use limn::util::*;
use limn::widget::text::*;
use limn::widget;

use limn::widget::builder::WidgetBuilder;
use limn::widget::primitives::{RectDrawable};
use limn::widget::button::ToggleEventHandler;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::event::{self, Event, Signal};
use limn::widget::{EventHandler, EventArgs};
use limn::resources::Id;
use limn::color::*;
use limn::eventbus::EventAddress;

use input::EventId;
use backend::glyph::GlyphCache;
use backend::{Window, WindowEvents};
use input::ResizeEvent;
use backend::events::WindowEvent;

use std::any::Any;

const COUNTER: EventId = EventId("COUNTER");
const COUNT: EventId = EventId("COUNT");
pub struct PushButtonBuilder<'a> {
    pub widget: WidgetBuilder,
    resources: &'a Resources,
}
impl<'a> PushButtonBuilder<'a> {
    pub fn new(resources: &'a Resources) -> Self {
        let rect = RectDrawable { background: RED };
        let mut widget = WidgetBuilder::new();
        widget.set_drawable(widget::primitives::draw_rect, Box::new(rect));

        widget.layout.dimensions(Dimensions { width: 100.0, height: 50.0 });
        
        PushButtonBuilder { widget: widget, resources: resources }
    }
    pub fn set_text(&mut self, text: &'static str, font_id: Id) {

        let button_text_drawable = TextDrawable { text: text.to_owned(), font_id: font_id, font_size: 20.0, text_color: BLACK, background_color: TRANSPARENT };
        let button_text_dims = button_text_drawable.measure_dims_no_wrap(self.resources);
        let mut button_text_widget = WidgetBuilder::new();
        button_text_widget.set_drawable(widget::text::draw_text, Box::new(button_text_drawable));
        button_text_widget.layout.dimensions(button_text_dims);
        button_text_widget.layout.center(&self.widget.layout);

        self.widget.add_child(Box::new(button_text_widget));
    }
    pub fn builder(self) -> WidgetBuilder {
        self.widget
    }
}

fn main() {
    let mut resources = Resources::new();

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/Hack/Hack-Regular.ttf");

    let font_id = resources.fonts.insert_from_file(font_path).unwrap();
    
    let mut root_widget = WidgetBuilder::new();
    let root_id = resources.widget_id();
    root_widget.set_id(root_id);
    {
        let mut linear_layout = LinearLayout::new(Orientation::Horizontal, &root_widget.layout);
        let mut left_spacer = WidgetBuilder::new();
        left_spacer.layout.width(50.0);
        linear_layout.add_widget(&mut left_spacer.layout);
        root_widget.add_child(Box::new(left_spacer));

        let text_drawable = TextDrawable { text: "0".to_owned(), font_id: font_id, font_size: 20.0, text_color: BLACK, background_color: WHITE };
        let text_dims = text_drawable.measure_dims_no_wrap(&resources);
        let mut text_widget = WidgetBuilder::new();
        text_widget.set_drawable(widget::text::draw_text, Box::new(text_drawable));
        text_widget.layout.width(80.0);
        text_widget.layout.height(text_dims.height);
        text_widget.layout.center_vertical(&root_widget.layout);
        linear_layout.add_widget(&mut text_widget.layout);

        struct CountHandler {}
        impl EventHandler for CountHandler {
            fn event_id(&self) -> EventId {
                COUNT
            }
            fn handle_event(&mut self, event_args: EventArgs) {
                let EventArgs { event, state, .. } = event_args;
                let state = state.unwrap();
                let state = state.downcast_mut::<TextDrawable>().unwrap();
                let count: &u32 = event.event_data().unwrap().downcast_ref().unwrap();
                state.text = format!("{}", count);
            }
        }
        text_widget.event_handlers.push(Box::new(CountHandler {}));

        let mut button_container = WidgetBuilder::new();
        linear_layout.add_widget(&mut button_container.layout);
        struct PushButtonHandler {
            receiver_id: Id,
        }
        impl EventHandler for PushButtonHandler {
            fn event_id(&self) -> EventId {
                event::WIDGET_PRESS
            }
            fn handle_event(&mut self, event_args: EventArgs) {
                let EventArgs { event_queue, .. } = event_args;
                let event = Signal::new(COUNTER);
                event_queue.push(EventAddress::IdAddress("SELF".to_owned(), self.receiver_id.0), Box::new(event));
            }
        }
        let mut button_widget = PushButtonBuilder::new(&resources);
        button_widget.set_text("Count", font_id);
        button_widget.widget.layout.center(&button_container.layout);
        button_widget.widget.layout.pad(50.0, &button_container.layout);
        button_widget.widget.event_handlers.push(Box::new(PushButtonHandler { receiver_id: root_id }));
        button_container.add_child(Box::new(button_widget.builder()));
        root_widget.add_child(Box::new(text_widget));
        root_widget.add_child(Box::new(button_container));
    }

    event!(CountEvent, u32);
    struct CounterHandler {
        count: u32,
    }
    impl CounterHandler {
        fn new() -> Self {
            CounterHandler { count: 0 }
        }
    }
    impl EventHandler for CounterHandler {
        fn event_id(&self) -> EventId {
            COUNTER
        }
        fn handle_event(&mut self, event_args: EventArgs) {
            let EventArgs { widget_id, event_queue, .. } = event_args;
            self.count += 1;
            let event = CountEvent::new(COUNT, self.count);
            event_queue.push(EventAddress::IdAddress("CHILDREN".to_owned(), widget_id.0), Box::new(event));
        }
    }
    root_widget.event_handlers.push(Box::new(CounterHandler::new()));


    let ui = &mut Ui::new();
    ui.set_root(root_widget, &mut resources);

    let window_dims = ui.get_root_dims();
    let mut window = Window::new("Limn counter demo", window_dims, Some(window_dims));
    let mut glyph_cache = GlyphCache::new(&mut window.context.factory, 512, 512);

    let mut events = WindowEvents::new();
    while let Some(event) = events.next(&mut window) {
        match event {
            WindowEvent::Input(event) => {
                if let Some(window_dims) = event.resize_args() {
                    window.window_resized();
                    ui.window_resized(&mut window, window_dims.into());
                }
                ui.handle_event(event.clone());
            },
            WindowEvent::Render => {
                window.draw_2d(|context, graphics| {
                    graphics::clear([0.8, 0.8, 0.8, 1.0], graphics);
                    ui.draw(&resources, &mut glyph_cache, context, graphics);
                });
            }
        }
    }
}
