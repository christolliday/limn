
use glutin;
use util::Point;

#[derive(Clone, Debug)]
pub struct KeyboardInput(pub glutin::ElementState, pub glutin::ScanCode, pub Option<glutin::VirtualKeyCode>);
#[derive(Debug)]
pub struct WidgetKeyboardInput(pub glutin::ElementState, pub glutin::ScanCode, pub Option<glutin::VirtualKeyCode>);

pub struct MouseMoved(pub Point);
pub struct MouseWheel(pub glutin::MouseScrollDelta);
pub struct MouseButton(pub glutin::ElementState, pub glutin::MouseButton);

pub struct WidgetMouseWheel(pub glutin::MouseScrollDelta);
pub struct WidgetMouseButton(pub glutin::ElementState, pub glutin::MouseButton);
