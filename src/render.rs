use std::sync::Arc;
use std::sync::atomic::{self, AtomicBool};

use gleam::gl;
use glutin;
use webrender;
use webrender::api::*;

use window::Window;
use euclid::TypedPoint2D;
use resources;
use geometry::{Rect, RectExt, Point, Size};

// Provides access to the WebRender context and API
pub(super) struct WebRenderContext {
    pub renderer: webrender::Renderer,
    pub render_api: RenderApi,
    pub epoch: Epoch,
    pub pipeline_id: PipelineId,
    pub document_id: DocumentId,
    pub device_pixel_ratio: f32,
    pub root_background_color: ColorF,
    // store frame ready event in case it is received after
    // update but before the event queue is waiting, otherwise
    // the event queue can go idle while there is a frame ready
    pub frame_ready: Arc<AtomicBool>,
}

// Context needed for widgets to draw or update resources in a particular frame
pub struct RenderBuilder {
    pub builder: DisplayListBuilder,
    pub resources: ResourceUpdates,
}

impl WebRenderContext {
    pub fn new(window: &mut Window, events_loop: &glutin::EventsLoop) -> Self {
        let gl = window.gl();
        println!("OpenGL version {}", gl.get_string(gl::VERSION));

        let opts = webrender::RendererOptions {
            resource_override_path: None,
            debug: true,
            precache_shaders: false,
            device_pixel_ratio: window.hidpi_factor(),
            .. webrender::RendererOptions::default()
        };

        let (renderer, sender) = webrender::Renderer::new(gl, opts).unwrap();
        let api = sender.create_api();
        resources::init_resources(sender.create_api());
        let document_id = api.add_document(window.device_size());

        let frame_ready = Arc::new(AtomicBool::new(false));
        let notifier = Box::new(Notifier::new(events_loop.create_proxy(), frame_ready.clone()));
        renderer.set_render_notifier(notifier);

        let epoch = Epoch(0);
        let root_background_color = ColorF::new(0.8, 0.8, 0.8, 1.0);

        let pipeline_id = PipelineId(0, 0);
        api.set_root_pipeline(document_id, pipeline_id);
        WebRenderContext {
            renderer: renderer,
            render_api: api,
            epoch: epoch,
            pipeline_id: pipeline_id,
            document_id: document_id,
            device_pixel_ratio: window.hidpi_factor(),
            root_background_color: root_background_color,
            frame_ready: frame_ready,
        }
    }
    pub fn deinit(self) {
        self.renderer.deinit();
    }
    pub fn render_builder(&mut self, window_size: LayoutSize) -> RenderBuilder {
        let builder = DisplayListBuilder::new(self.pipeline_id, window_size);
        RenderBuilder {
            builder: builder,
            resources: ResourceUpdates::new(),
        }
    }
    pub fn set_display_list(&mut self, builder: DisplayListBuilder, resources: ResourceUpdates, window_size: LayoutSize) {
        self.render_api.set_display_list(
            self.document_id,
            self.epoch,
            Some(self.root_background_color),
            window_size,
            builder.finalize(),
            true,
            resources
        );
    }
    pub fn generate_frame(&mut self) {
        self.render_api.generate_frame(self.document_id, None);
    }
    pub fn frame_ready(&mut self) -> bool {
        self.frame_ready.load(atomic::Ordering::Acquire)
    }
    // if there is a frame ready, update current frame and render it, otherwise, does nothing
    pub fn update(&mut self, window_size: DeviceUintSize) {
        self.frame_ready.store(false, atomic::Ordering::Release);
        self.renderer.update();
        self.renderer.render(window_size).unwrap();
    }
    pub fn toggle_flags(&mut self, toggle_flags: webrender::DebugFlags) {
        let mut flags = self.renderer.get_debug_flags();
        flags.toggle(toggle_flags);
        self.renderer.set_debug_flags(flags);
    }
    pub fn window_resized(&mut self, size: DeviceUintSize) {
        let window_rect = DeviceUintRect::new(TypedPoint2D::zero(), size);
        self.render_api.set_window_parameters(self.document_id, size, window_rect, self.device_pixel_ratio);
    }
}

struct Notifier {
    events_proxy: glutin::EventsLoopProxy,
    frame_ready: Arc<AtomicBool>,
}
impl Notifier {
    fn new(events_proxy: glutin::EventsLoopProxy, frame_ready: Arc<AtomicBool>) -> Self {
        Notifier {
            events_proxy: events_proxy,
            frame_ready: frame_ready,
        }
    }
}

impl RenderNotifier for Notifier {
    fn new_frame_ready(&mut self) {
        #[cfg(not(target_os = "android"))]
        debug!("new frame ready");
        self.events_proxy.wakeup().ok();
        self.frame_ready.store(true, atomic::Ordering::Release);
    }

    fn new_scroll_frame_ready(&mut self, _composite_needed: bool) {
        #[cfg(not(target_os = "android"))]
        debug!("new scroll frame ready");
        self.events_proxy.wakeup().ok();
    }
}

pub fn draw_rect_outline<C: Into<ColorF>>(rect: Rect, color: C, renderer: &mut RenderBuilder) {
    let widths = BorderWidths { left: 1.0, right: 1.0, top: 1.0, bottom: 1.0 };
    let side = BorderSide { color: color.into(), style: BorderStyle::Solid };
    let border = NormalBorder { left: side, right: side, top: side, bottom: side, radius: BorderRadius::zero() };
    let details = BorderDetails::Normal(border);
    let info = PrimitiveInfo::new(rect.typed());
    renderer.builder.push_border(&info, widths, details);
}

pub fn draw_horizontal_line<C: Into<ColorF>>(baseline: f32, start: f32, end: f32, color: C, renderer: &mut RenderBuilder) {
    draw_rect_outline(Rect::new(Point::new(start, baseline), Size::new(end - start, 0.0)), color, renderer);
}
