pub fn setup_shader(shader: &str, s_type: gl::types::GLenum) -> u32 {
    let shader_vec = vec![shader];
    let size: Vec<i32> = shader_vec.iter().map(|y| y.len() as i32).collect();

    let id = gl::CreateShader(s_type);
    gl::ShaderSource(id, 1, shader_vec, &size[..]);
    gl::CompileShader(id);
    id
}

pub fn apply_shaders(shader: Vec<u32>) -> u32 {
    let program = gl::CreateProgram();
    shader.iter().for_each(|&s| gl::AttachShader(program, s));
    gl::LinkProgram(program);
    program
}
