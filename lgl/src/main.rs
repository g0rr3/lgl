use std::mem::size_of;

use gl::{
    self, types::GLsizeiptr, AttachShader, BindBuffer, BindVertexArray, BufferData, Clear,
    ClearColor, CompileShader, CreateProgram, CreateShader, DrawArrays, EnableVertexAttribArray,
    GenBuffers, GenVertexArrays, GetAttribLocation, LinkProgram, ShaderSource, UseProgram,
    VertexAttribPointer, FRAGMENT_SHADER, VERTEX_SHADER,
};
use glutin::event::{Event, WindowEvent};

mod debug;

static VERTEX_DATA: [f32; 15] = [
    -0.5, -0.5, 1.0, 0.0, 0.0, 0.0, 0.5, 0.0, 1.0, 0.0, 0.5, -0.5, 0.0, 0.0, 1.0,
];

fn main() {
    let vs_src: Vec<&str> = vec![include_str!("shader/test.vertexshader")];
    let fs_src: Vec<&str> = vec![include_str!("shader/test.fragmentshader")];
    let _car_color: i32 = 0;

    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("Men");
    let w_context = glutin::ContextBuilder::new()
        .build_windowed(wb, &el)
        .unwrap();
    let w_context = unsafe { w_context.make_current().unwrap() };

    gl::load_with(|ptr| w_context.get_proc_address(ptr) as *const _);
    unsafe { debug::enable_gl_debug(debug::GLErrorSeverityLogLevel::DEBUG_SEVERITY_HIGH) };

    let a: Vec<i32> = vs_src.iter().map(|x| x.len() as i32).collect();
    let b: Vec<i32> = fs_src.iter().map(|x| x.len() as i32).collect();

    let vs = CreateShader(VERTEX_SHADER);
    ShaderSource(vs, 1, vs_src, &a[..]);
    CompileShader(vs);

    let fs = CreateShader(FRAGMENT_SHADER);
    ShaderSource(fs, 1, fs_src, &b[..]);
    CompileShader(fs);

    let program = CreateProgram();
    AttachShader(program, vs);
    AttachShader(program, fs);
    LinkProgram(program);
    UseProgram(program);

    let vb = GenBuffers(1);
    BindBuffer(gl::ARRAY_BUFFER, vb);
    BufferData(
        gl::ARRAY_BUFFER,
        (VERTEX_DATA.len() * size_of::<f32>()) as GLsizeiptr,
        VERTEX_DATA.as_ptr() as *const _,
        gl::STATIC_DRAW,
    );

    let va = GenVertexArrays(1);
    BindVertexArray(va);

    let pos_attrib = GetAttribLocation(program, "position");
    let color_attrib = GetAttribLocation(program, "color");

    VertexAttribPointer(
        pos_attrib as u32,
        2,
        gl::FLOAT,
        0,
        5 * size_of::<f32>() as i32,
        std::ptr::null(),
    );

    VertexAttribPointer(
        color_attrib as u32,
        3,
        gl::FLOAT,
        0,
        5 * std::mem::size_of::<f32>() as i32,
        (2 * std::mem::size_of::<f32>()) as *const () as *const _,
    );

    EnableVertexAttribArray(pos_attrib as u32);
    EnableVertexAttribArray(color_attrib as u32);

    el.run(move |event, _, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => w_context.resize(new_size),
                WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                ClearColor(0.2, 0.3, 0.3, 1.0);
                Clear(gl::COLOR_BUFFER_BIT);
                DrawArrays(gl::TRIANGLES, 0, 3);

                w_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
