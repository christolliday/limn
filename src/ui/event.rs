
use glutin;
use util::Point;

pub struct MouseMoved(pub Point);
pub struct MouseWheel(pub glutin::MouseScrollDelta);
pub struct MouseButton(pub glutin::ElementState, pub glutin::MouseButton);

pub struct WidgetMouseWheel(pub glutin::MouseScrollDelta);
pub struct WidgetMouseButton(pub glutin::ElementState, pub glutin::MouseButton);
