extern crate limn;
#[macro_use]
extern crate limn_layout;
#[macro_use]
extern crate lazy_static;
extern crate text_layout;
extern crate env_logger;
extern crate glutin;
extern crate gleam;
extern crate euclid;

mod util;

use std::cell::Cell;
use std::rc::Rc;
use std::iter::once;

use gleam::gl::{self, GLushort, GLint, GLuint, GLfloat};
use euclid::{Vector3D, Transform3D};

use limn::prelude::*;

use limn::app::App;
use limn::window::Window;
use limn::input::{EscKeyCloseHandler, DebugSettingsHandler};
use limn::widgets::slider::{SliderBuilder, SliderEvent};
use limn::widgets::glcanvas::{GLCanvasBuilder, GLCanvasState};

fn init_framebuffer(gl: &Rc<gl::Gl>) -> (GLuint, GLuint, GLuint) {
    // Make a texture that will be sent to WebRender
    let tex = gl.gen_textures(1)[0];

    // Make a renderbuffer for the depth test
    let depth_buf = gl.gen_renderbuffers(1)[0];

    // Make a framebuffer object (render target for our demo)
    let fb = gl.gen_framebuffers(1)[0];
    gl.bind_framebuffer(gl::FRAMEBUFFER, fb);

    // Set them up
    resize_destination(gl, tex, depth_buf, 1024, 768);

    // Unbind the framebuffer (so that WebRender will render to the window)
    gl.bind_framebuffer(gl::FRAMEBUFFER, 0);

    (fb, tex, depth_buf)
}

fn resize_destination(gl: &Rc<gl::Gl>, tex: GLuint, depth_buf: GLuint, width: GLint, height: GLint) {
    gl.bind_texture(gl::TEXTURE_2D, tex);
    gl.tex_image_2d(gl::TEXTURE_2D, 0, gl::RGB as _, width, height, 0, gl::RGB, gl::UNSIGNED_BYTE, None);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);

    gl.bind_renderbuffer(gl::RENDERBUFFER, depth_buf);
    gl.renderbuffer_storage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8 as _, width, height);

    gl.framebuffer_texture_2d(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, tex, 0);
    gl.framebuffer_renderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, depth_buf);
}

fn init_boxes_model(gl: &Rc<gl::Gl>) -> (GLuint, usize) {
    let vao = gl.gen_vertex_arrays(1)[0];
    gl.bind_vertex_array(vao);

    // Mesh vertices (indexed), set as attribute 0 for the fragment shader
    gl.enable_vertex_attrib_array(0);

    let vert_buf = gl.gen_buffers(1)[0];
    gl.bind_buffer(gl::ARRAY_BUFFER, vert_buf);
    let verts : [GLfloat; 24] = [
        -1.0, 1.0, 1.0,
        -1.0, 1.0,-1.0,
        -1.0,-1.0, 1.0,
        -1.0,-1.0,-1.0,
        1.0,  1.0, 1.0,
        1.0,  1.0,-1.0,
        1.0, -1.0, 1.0,
        1.0, -1.0,-1.0,
    ];
    gl::buffer_data(&**gl, gl::ARRAY_BUFFER, &verts[..], gl::STATIC_DRAW);

    let idx_buf = gl.gen_buffers(1)[0];
    gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, idx_buf);
    let inds : [GLushort; 36] = [
        2, 6, 4,  4, 0, 2,  4, 5, 7,  7, 6, 4,  3, 7, 5,  5, 1, 3,  3, 1, 0,  0, 2, 3,  4, 0, 1,  1, 5, 4,  3, 7, 6,  6, 2, 3
    ];
    gl::buffer_data(&**gl, gl::ELEMENT_ARRAY_BUFFER, &inds[..], gl::STATIC_DRAW);

    gl.vertex_attrib_pointer_f32(0, 3, false, 0, 0);
    gl.vertex_attrib_divisor(0, 0);

    // Positions for instancing, set as attribute 1 for the fragment shader
    gl.enable_vertex_attrib_array(1);

    let pos_buf = gl.gen_buffers(1)[0];
    gl.bind_buffer(gl::ARRAY_BUFFER, pos_buf);
    let positions = (-10..10).flat_map(|x|
        (-10..10).flat_map(move |z| once(x as GLfloat * 1.3).chain(once(0.0)).chain(once(z as GLfloat * 1.5)))
    ).collect::<Vec<GLfloat>>();
    gl::buffer_data(&**gl, gl::ARRAY_BUFFER, &positions[..], gl::STATIC_DRAW);

    gl.vertex_attrib_pointer_f32(1, 3, false, 0, 0);
    gl.vertex_attrib_divisor(1, 1);

    (vao, positions.len() / 3)
}

fn init_boxes_shader(gl: &Rc<gl::Gl>) -> GLuint {
    let vert_shader = gl.create_shader(gl::VERTEX_SHADER);
    let vert_shader_src = b"
        #version 330 core
        uniform float scale;
        uniform float time;
        uniform mat4 view;
        uniform mat4 projection;
        layout (location = 0) in vec3 vertex;
        layout (location = 1) in vec3 position;
        out vec3 cposition;

        mat3 rotate(vec3 axis, float a) {
            float c = cos(a);
            vec3 as = axis * sin(a);
            mat3 p = mat3(axis.x * axis, axis.y * axis, axis.z * axis);
            mat3 q = mat3(c, -as.z, as.y, as.z, c, -as.x, -as.y, as.x, c);
            return p * (1.0 - c) + q;;
        }

        void main() {
            gl_Position = projection * view * vec4(rotate(vec3(0.2, 0.3, 1), time) * vertex * scale + position, 1);
            cposition = gl_Position.xyz;
        }";
    gl.shader_source(vert_shader, &[&vert_shader_src[..]]);
    gl.compile_shader(vert_shader);

    let frag_shader = gl.create_shader(gl::FRAGMENT_SHADER);
    let frag_shader_src = b"
        #version 330 core
        uniform float time;
        in vec3 cposition;
        out vec4 color;

        void main() {
            vec3 normal = normalize(cross(dFdx(cposition), dFdy(cposition.xyz)));
            color = vec4(
                0.1 + normal.z * 0.3 + cposition.x * 0.1,
                0.3 + normal.y * 0.5 + normal.z * 0.1,
                0.4 + normal.z * 0.4,
                1
            );
        }";
    gl.shader_source(frag_shader, &[&frag_shader_src[..]]);
    gl.compile_shader(frag_shader);

    let prog = gl.create_program();
    gl.attach_shader(prog, vert_shader);
    gl.attach_shader(prog, frag_shader);
    gl.link_program(prog);
    gl.delete_shader(vert_shader);
    gl.delete_shader(frag_shader);

    prog
}

fn perspective_matrix(fovy: GLfloat, aspect: GLfloat, near: GLfloat, far: GLfloat) -> Transform3D<GLfloat> {
    let tan_half_fovy = (fovy / 2.0).tan();
    Transform3D::row_major(
        1.0 / (aspect * tan_half_fovy), 0.0, 0.0, 0.0,
        0.0, 1.0 / tan_half_fovy, 0.0, 0.0,
        0.0, 0.0, (far + near) / (near - far), -1.0,
        0.0, 0.0, - (2.0 * far * near) / (far - near), 0.0
    )
}

fn view_matrix(eye: Vector3D<GLfloat>, center: Vector3D<GLfloat>) -> Transform3D<GLfloat> {
    let f = (center - eye).normalize();
    let s = f.cross(Vector3D::new(0.0, 1.0, 0.0)).normalize();
    let u = s.cross(f);
    Transform3D::column_major(
        s.x, s.y, s.z, -s.dot(eye),
        u.x, u.y, u.z, -u.dot(eye),
        -f.x, -f.y, -f.z, f.dot(eye),
        0.0, 0.0, 0.0, 1.0
    )
}

lazy_static! {
    static ref VIEW_MATRIX: Transform3D<GLfloat> = view_matrix(Vector3D::new(0.0, 5.0, 2.0), Vector3D::new(0.0, 0.4, 0.0));
}

#[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
fn draw_boxes(gl: &Rc<gl::Gl>, prog: GLuint, model: GLuint,
              width: GLint, height: GLint, box_count: usize,
              time_cell: &Cell<f32>, scale_cell: &Cell<f32>) {
    gl.viewport(0, 0, width, height);
    gl.clear_color(0.3, 0.3, 0.3, 1.0);
    gl.clear_depth(1.0);
    gl.cull_face(gl::BACK);
    gl.enable(gl::DEPTH_TEST);
    gl.depth_mask(true);
    gl.depth_func(gl::LESS);
    gl.depth_range(0.0, 1.0);
    gl.use_program(prog);
    gl.bind_vertex_array(model);

    let projection = perspective_matrix(1.30899, width as GLfloat / height as GLfloat, 0.15, 10.0).to_row_major_array();
    let view = (*VIEW_MATRIX).to_row_major_array();
    gl.uniform_matrix_4fv(gl.get_uniform_location(prog, "view"), false, &view[..]);
    gl.uniform_matrix_4fv(gl.get_uniform_location(prog, "projection"), false, &projection[..]);
    gl.uniform_1f(gl.get_uniform_location(prog, "scale"), scale_cell.get());
    let time = time_cell.get();
    gl.uniform_1f(gl.get_uniform_location(prog, "time"), time);
    time_cell.set(time + 0.01);

    gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    gl.draw_elements_instanced(gl::TRIANGLE_STRIP, 36, gl::UNSIGNED_SHORT, 0, box_count as _);
}

fn main() {
    env_logger::init().unwrap();
    let window_size = (1280, 720);
    let events_loop = glutin::EventsLoop::new();
    let window = Window::new("Limn+OpenGL boxes demo", window_size, Some(window_size), &events_loop);

    // This returns a gleam struct. But if you look at the source of .gl(), you
    // can figure out how to connect any other OpenGL wrapper.
    let gl = window.gl();

    // Prepare all the GL resources
    let (fb, tex, depth_buf) = init_framebuffer(&gl);
    let prog = init_boxes_shader(&gl);
    let (model, box_count) = init_boxes_model(&gl);

    // Setup the app
    let mut app = App::new(window, events_loop);
    app.add_handler(EscKeyCloseHandler);
    app.add_handler(DebugSettingsHandler::new());
    let mut root = WidgetBuilder::new("root");

    // Create an image that's connected to the texture we're rendering to
    let mut gl_canvas = GLCanvasBuilder::new("gl_canvas", u64::from(tex));
    gl_canvas.layout().no_container();
    gl_canvas.layout().add(constraints![
        match_width(&root),
        min_height(480.0),
    ]);

    // Setup state
    let time = Rc::new(Cell::new(0.01));
    let scale = Rc::new(Cell::new(0.8));
    let target_size = Rc::new(Cell::new((0.0, 0.0)));

    // Render frames in the app loop
    let scale_c = Rc::clone(&scale); // The slider will take the original
    let mut gl_canvas_ref = gl_canvas.widget_ref();
    app.add_handler_fn(move |_: &FrameEvent, args| {
        gl.bind_framebuffer(gl::FRAMEBUFFER, fb);

        // Handle widget size changes
        if let Some(state) = gl_canvas_ref.draw_state().downcast_ref::<GLCanvasState>() {
            let (old_size_width, old_size_height) = target_size.get();
            let new_size = state.measure();
            if new_size.width != old_size_width || new_size.height != old_size_height {
                target_size.set((new_size.width, new_size.height));
                resize_destination(&gl, tex, depth_buf, new_size.width as _, new_size.height as _);
            }
        }

        // Draw!
        let (width, height) = target_size.get();
        draw_boxes(&gl, prog, model, width as _, height as _, box_count, &time, &scale_c);

        gl.bind_framebuffer(gl::FRAMEBUFFER, 0);
        gl_canvas_ref.set_updated(true);

        args.ui.redraw();
    });

    // And now the slider
    let mut size_slider = SliderBuilder::new();
    size_slider
        .set_range(0.8..1.2)
        .set_value(scale.get())
        .set_name("size_slider");
    size_slider.layout().add(constraints![
        align_below(&gl_canvas).padding(10.0),
        align_left(&root).padding(10.0),
        align_right(&root).padding(10.0),
        align_bottom(&root).padding(10.0),
    ]);
    size_slider.add_handler_fn(move |event: &SliderEvent, _| {
        scale.set(event.value);
    });

    root.add_child(gl_canvas);
    root.add_child(size_slider);
    app.main_loop(root);
}
