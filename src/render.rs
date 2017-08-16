use gleam::gl;
use glutin;
use webrender;
use webrender::api::*;

use window::Window;
//use util::Point;
use euclid::TypedPoint2D;

pub struct WebRenderContext {
    renderer: webrender::Renderer,
    pub render_api: RenderApi,
    pub epoch: Epoch,
    pub pipeline_id: PipelineId,
    pub document_id: DocumentId,
    pub root_background_color: ColorF,
}

pub struct RenderBuilder<'a> {
    pub render: &'a WebRenderContext,
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
            precache_shaders: true,
            device_pixel_ratio: window.hidpi_factor(),
            .. webrender::RendererOptions::default()
        };

        let (renderer, sender) = webrender::renderer::Renderer::new(gl, opts).unwrap();
        let api = sender.create_api();
        let document_id = api.add_document(window.size_u32());

        let notifier = Box::new(Notifier { events_proxy: events_loop.create_proxy() });
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
            root_background_color: root_background_color,
        }
    }
    pub fn deinit(self) {
        self.renderer.deinit();
    }
    pub fn render_builder(&mut self, window_size: LayoutSize) -> RenderBuilder {
        let builder = DisplayListBuilder::new(self.pipeline_id, window_size);
        RenderBuilder {
            render: self,
            builder: builder,
            resources: ResourceUpdates::new(),
        }
    }
    pub fn update(&mut self, builder: DisplayListBuilder, resources: ResourceUpdates, window_size: LayoutSize) {
        self.render_api.set_display_list(
            self.document_id,
            self.epoch,
            Some(self.root_background_color),
            window_size,
            builder.finalize(),
            true,
            resources
        );
        self.render_api.generate_frame(self.document_id, None);
    }
    pub fn render(&mut self, window_size: DeviceUintSize) {
        self.renderer.update();
        self.renderer.render(window_size);
        //window.swap_buffers();
    }
    pub fn toggle_flags(&mut self, toggle_flags: webrender::renderer::DebugFlags) {
        let mut flags = self.renderer.get_debug_flags();
        flags.toggle(toggle_flags);
        self.renderer.set_debug_flags(flags);
    }
    pub fn window_resized(&mut self, size: DeviceUintSize) {
        let window_rect = DeviceUintRect::new(TypedPoint2D::zero(), size);
        self.render_api.set_window_parameters(self.document_id, size, window_rect);
    }
}

struct Notifier {
    events_proxy: glutin::EventsLoopProxy,
}
impl Notifier {
    fn new(events_proxy: glutin::EventsLoopProxy) -> Self {
        Notifier {
            events_proxy: events_proxy,
        }
    }
}

impl RenderNotifier for Notifier {
    fn new_frame_ready(&mut self) {
        #[cfg(not(target_os = "android"))]
        self.events_proxy.wakeup().ok();
    }

    fn new_scroll_frame_ready(&mut self, _composite_needed: bool) {
        #[cfg(not(target_os = "android"))]
        self.events_proxy.wakeup().ok();
    }
}
