use sdl2::video::{GLProfile, Window};
use std::rc::Rc;

use cardboard_math::*;
use cardboard_oogl::*;

// <https://github.com/gfx-rs/wgpu-rs/blob/2ef725065e68164cced1551c7a2540523eb0ca77/examples/framework.rs#L336-L337>
#[allow(dead_code)]
fn main() {}

pub fn prepare_example_gl_context(
  example_name: &'static str,
  window_size: Vec2u32,
) -> (sdl2::Sdl, sdl2::VideoSubsystem, Window, sdl2::EventPump, SharedContext) {
  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();

  let gl_attr = video_subsystem.gl_attr();
  gl_attr.set_context_profile(GLProfile::GLES);
  gl_attr.set_context_version(2, 0);

  let window_title = format!("{} {} example", env!("CARGO_PKG_NAME"), example_name);
  let window = video_subsystem
    .window(&window_title, window_size.x, window_size.y)
    .resizable()
    .opengl()
    .allow_highdpi()
    .build()
    .unwrap();

  let sdl_gl_ctx = window.gl_create_context().unwrap();
  let gl = Rc::new(Context::load_with(&video_subsystem, sdl_gl_ctx));

  let event_pump = sdl_context.event_pump().unwrap();

  (sdl_context, video_subsystem, window, event_pump, gl)
}

pub fn reset_gl_viewport(gl: &Context, window: &Window) {
  let (w, h) = window.drawable_size();
  gl.set_viewport(vec2(0, 0), vec2(w as i32, h as i32));
}

pub fn compile_shader(gl: SharedContext, src: &str, type_: ShaderType) -> Shader {
  let shader = Shader::new(gl, type_);
  shader.set_source(src.as_bytes());
  let success = shader.compile();

  let log = shader.get_info_log();
  let log = String::from_utf8_lossy(&log);
  if !success {
    panic!("Shader compilation error(s):\n{}", log);
  } else if !log.is_empty() {
    eprintln!("Shader compilation warning(s):\n{}", log);
  }

  shader
}

pub fn link_program(gl: SharedContext, vertex: &Shader, fragment: &Shader) -> Program {
  let program = Program::new(gl);
  program.attach_shader(vertex);
  program.attach_shader(fragment);
  let success = program.link();

  let log = program.get_info_log();
  let log = String::from_utf8_lossy(&log);
  if !success {
    panic!("Program linking error(s):\n{}", log);
  } else if !log.is_empty() {
    eprintln!("Program linking warning(s):\n{}", log);
  }

  program.detach_shader(vertex);
  program.detach_shader(fragment);
  program.load_descriptors();
  program
}
