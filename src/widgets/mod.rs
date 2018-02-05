//! Includes standard bundled widgets.

pub mod button;
pub mod scroll;
pub mod list;
pub mod slider;
pub mod edit_text;
pub mod image;
pub mod glcanvas;
pub mod text;

pub mod prelude {
    pub use super::text::StaticTextStyle;
    pub use super::button::{ButtonStyle, ToggleButtonStyle, ToggleEvent};
    pub use super::edit_text::{EditText, TextUpdated};
    pub use super::slider::{Slider, SetSliderValue, SliderEvent};
    pub use super::list::{List, ListItemSelected, ItemSelected, ListItemHandler};
    pub use super::scroll::ScrollContainer;
    pub use super::image::Image;
    pub use super::glcanvas::{GLCanvasBuilder, GLCanvasState};
}
