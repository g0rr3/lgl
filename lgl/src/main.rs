use std::{mem::size_of, thread::sleep_ms};

use gl::{
    self, BindBuffer, BindVertexArray, BufferData, ClearColor, DrawArrays, EnableVertexAttribArray,
    GenBuffers, GenVertexArrays, GetUniformLocation, TexImage2D, Uniform4f, VertexAttribPointer,
};
use glutin::event::{Event, WindowEvent};
use rand::prelude::*;

mod debug;
mod helper;
mod tester;

#[rustfmt::skip]
static VERTEX_DATA: &[f32] = &[
    0.0,  0.5, 0.0,
    0.5, -0.5, 0.0,
   -0.5, -0.5, 0.0,  
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

    let mut rng = rand::thread_rng();

    let program = helper::apply_shaders(vec![fs, vs]);
    let my_color = GetUniformLocation(program, "myColor");

    let va = GenVertexArrays(1);
    BindVertexArray(va);

    let vb = GenBuffers(1);
    BindBuffer(gl::ARRAY_BUFFER, vb);
    BufferData(
        gl::ARRAY_BUFFER,
        (VERTEX_DATA.len() * size_of::<f32>()) as isize,
        VERTEX_DATA.as_ptr() as *const _,
        gl::STATIC_DRAW,
    );

    VertexAttribPointer(
        0,
        3,
        gl::FLOAT,
        0,
        3 * size_of::<f32>() as i32,
        std::ptr::null(),
    );

    EnableVertexAttribArray(0);

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
                ClearColor(0.0, 0.0, 0.0, 0.0);
                let red: f32 = rng.gen_range(0.0..=1.0);
                let blue: f32 = rng.gen_range(0.0..=1.0);
                let green: f32 = rng.gen_range(0.0..=1.0);

                Uniform4f(my_color, red, green, blue, 1.0);

                DrawArrays(gl::TRIANGLES, 0, 3);

                w_context.swap_buffers().unwrap();
                sleep_ms(300);
            }
            _ => (),
        }
    });
}
