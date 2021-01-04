use std::{mem::size_of, thread::sleep_ms};

use gl::{
    self, BindBuffer, BindTexture, BindVertexArray, BufferData, ClearColor, DrawArrays,
    EnableVertexAttribArray, GenBuffers, GenTextures, GenVertexArrays, GenerateMipmap,
    GetUniformLocation, PixelStorei, TexImage2D, TexParameteri, Uniform4f, VertexAttribPointer,
};
use glutin::event::{Event, WindowEvent};
use rand::prelude::*;

mod debug;
mod helper;
mod tester;

#[rustfmt::skip]
static VERTICES: &[f32] = &[
    // positions          // colors           // texture coords
     0.5,  0.5, 0.0,   1.0, 0.0, 0.0,   1.0, 1.0,   // top right
     0.5, -0.5, 0.0,   0.0, 1.0, 0.0,   1.0, 0.0,   // bottom right
    -0.5, -0.5, 0.0,   0.0, 0.0, 1.0,   0.0, 0.0,   // bottom left
    -0.5,  0.5, 0.0,   1.0, 1.0, 0.0,   0.0, 1.0    // top left 
];

#[rustfmt::skip]
static _TEX_COORDS: &[f32] = &[
    0.0, 0.0,
    1.0, 0.0,
    0.5, 1.0 
];

const VS_SRC: &str = include_str!("shader/test.vert");
const FS_SRC: &str = include_str!("shader/test.frag");

fn main() {
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("Men");
    let w_context = glutin::ContextBuilder::new()
        .build_windowed(wb, &el)
        .unwrap();
    let w_context = unsafe { w_context.make_current().unwrap() };

    gl::load_with(|ptr| w_context.get_proc_address(ptr) as *const _);
    unsafe { debug::enable_gl_debug(debug::GLErrorSeverityLogLevel::DEBUG_SEVERITY_HIGH) };
    let vs = helper::setup_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = helper::setup_shader(FS_SRC, gl::FRAGMENT_SHADER);

    PixelStorei(gl::UNPACK_ALIGNMENT, 1);

    let image = image::open("lgl/wall.jpg").expect("Failed to load image");

    let image_buffer = image.as_rgb8().expect("Provided image not in RGB8 format");

    let image_data = image_buffer.as_raw();
    let image_data_len = image_data.len();

    let texture = GenTextures(1);

    BindTexture(gl::TEXTURE_2D, texture);
    // set the texture wrapping/filtering options (on the currently bound texture object)
    TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
    TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);
    TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
    TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);

    TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGB as _,
        image_buffer.width() as _,
        image_buffer.height() as _,
        0,
        gl::RGB,
        gl::UNSIGNED_BYTE, // 0..=255
        image_data.as_ptr() as *const _,
    );
    GenerateMipmap(gl::TEXTURE_2D);

    let program = helper::apply_shaders(vec![fs, vs]);

    // bind texture to ouruniform
    gl::UseProgram(program);

    let texture_slot = 3;
    let texture_uniform_location = gl::GetUniformLocation(program, "ourTexture");
    gl::Uniform1i(texture_uniform_location, texture_slot);

    let va = GenVertexArrays(1);
    BindVertexArray(va);

    let vb = GenBuffers(1);
    BindBuffer(gl::ARRAY_BUFFER, vb);
    BufferData(
        gl::ARRAY_BUFFER,
        (VERTICES.len() * size_of::<f32>()) as isize,
        VERTICES.as_ptr() as *const _,
        gl::STATIC_DRAW,
    );

    let position_location = 0;
    let position_offset = 0;
    let colors_location = 1;
    let colors_offset = 3 * size_of::<f32>();
    let uv_location = 2;
    let uv_offset = colors_offset + 3 * size_of::<f32>();
    // PPPCCCTT...
    let stride = (3 + 3 + 2) * size_of::<f32>();
    // location 0 (vertex position)
    VertexAttribPointer(
        position_location,
        3,
        gl::FLOAT,
        0,
        stride as i32,
        position_offset as usize as *const _,
    );
    EnableVertexAttribArray(position_location);

    // location 1 (texture coordinate)
    VertexAttribPointer(
        colors_location,
        3,
        gl::FLOAT,
        0,
        stride as i32,
        colors_offset as usize as *const _,
    );
    EnableVertexAttribArray(colors_location);

    // location 2 (texture coordinate)
    VertexAttribPointer(
        uv_location,
        2,
        gl::FLOAT,
        0,
        stride as i32,
        uv_offset as usize as *const _,
    );
    EnableVertexAttribArray(uv_location);

    el.run(move |event, _, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => w_context.resize(new_size),
                WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                gl::UseProgram(program);

                // binds our texture with handle `texture` to to texture slot GL_TEXTURE<texture_slot>
                {
                    gl::ActiveTexture(gl::TEXTURE0 + texture_slot as u32);
                    gl::BindTexture(gl::TEXTURE_2D, texture);
                }

                DrawArrays(gl::TRIANGLES, 0, 3);

                w_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
