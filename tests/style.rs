/// These tests are not parallelizable (resources/theme access not thread safe yet), use RUST_TEST_THREADS=1 cargo test for now

#[macro_use]
extern crate limn;

use limn::prelude::*;

component_style!{pub struct TestState<name="test", style=TestStyle> {
    text_a: String = String::from("default"),
    text_b: String = String::from("default"),
}}

impl Draw for TestState {
    fn draw(&mut self, _: Rect, _: Rect, _: &mut RenderBuilder) {}
}

fn setup() {
    resources().theme.register_type_style(TestStyle::default());
}

#[test]
fn style_class() {
    setup();

    resources().theme.register_class_style("test", style!(TestStyle {
        text_a: "test".to_owned(),
    }));

    let mut state = DrawState::default();
    state.set_draw_style(DrawStyle::from_class::<TestStyle>("test"));
    state.update(btreeset!{});

    let state = state.get_state::<TestState>();
    assert!(state.text_a == "test");
    assert!(state.text_b == "default");
}

#[test]
fn style_instance() {
    setup();

    let mut state = DrawState::default();
    state.set_draw_style(DrawStyle::from(style!(TestStyle {
        text_a: "test".to_owned(),
    })));
    state.update(btreeset!{});

    let state = state.get_state::<TestState>();
    assert!(state.text_a == "test");
    assert!(state.text_b == "default");
}
