use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::{GLProfile, Window};

use cardboard_math::*;
use cardboard_oogl::*;
use std::rc::Rc;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Vertex {
  pos: Vec2f,
  color: Colorf,
}

#[rustfmt::skip]
const VERTEX_DATA: [Vertex; 3] = [
  Vertex { pos: vec2( 0.0,  0.5), color: color(1.0, 0.0, 0.0, 1.0) },
  Vertex { pos: vec2( 0.5, -0.5), color: color(0.0, 1.0, 0.0, 1.0) },
  Vertex { pos: vec2(-0.5, -0.5), color: color(0.0, 0.0, 1.0, 1.0) },
];

static VS_SRC: &str = r#"#version 100
  attribute vec2 a_pos;
  attribute vec4 a_color;
  varying   vec4 v_color;
  void main(void) {
    gl_Position = vec4(a_pos, 0.0, 1.0);
    v_color = a_color;
  }
"#;

static FS_SRC: &str = r#"#version 100
  varying highp vec4 v_color;
  void main() {
    gl_FragColor = v_color;
  }
"#;

fn main() {
  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();

  let gl_attr = video_subsystem.gl_attr();
  gl_attr.set_context_profile(GLProfile::GLES);
  gl_attr.set_context_version(2, 0);

  let window = video_subsystem
    .window("cardboard_oogl triangle example", 800, 600)
    .resizable()
    .opengl()
    .allow_highdpi()
    .build()
    .unwrap();

  let sdl_gl_ctx = window.gl_create_context().unwrap();
  let gl = Rc::new(Context::load_with(&video_subsystem, sdl_gl_ctx));

  let mut event_pump = sdl_context.event_pump().unwrap();

  let vs = compile_shader(gl.share(), VS_SRC, ShaderType::Vertex);
  let fs = compile_shader(gl.share(), FS_SRC, ShaderType::Fragment);

  let mut program = link_program(gl.share(), &vs, &fs);
  let attr_pos = program.get_attrib::<Vec2f>("a_pos");
  let attr_color = program.get_attrib::<Colorf>("a_color");
  let bound_program = program.bind();

  let mut vbo = VertexBuffer::<Vertex>::new(
    gl.share(),
    vec![attr_pos.to_pointer_simple(), attr_color.to_pointer_simple()],
  );

  let bound_vbo = vbo.bind();
  bound_vbo.enable_attribs();
  bound_vbo.configure_attribs();
  bound_vbo.reserve_and_set(BufferUsageHint::StaticDraw, &VERTEX_DATA);

  gl.set_clear_color(color(0.0, 0.0, 0.0, 1.0));

  fn reset_viewport(gl: &Context, window: &Window) {
    let (w, h) = window.drawable_size();
    gl.set_viewport(vec2(0, 0), vec2(w as i32, h as i32));
  }

  reset_viewport(&gl, &window);

  'running: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
          break 'running;
        }

        Event::Window { win_event: WindowEvent::SizeChanged(..), .. } => {
          reset_viewport(&gl, &window);
        }

        _ => {}
      }
    }

    gl.clear(ClearFlags::COLOR);
    bound_vbo.draw(&bound_program, DrawPrimitive::Triangles);

    window.gl_swap_window();
  }
}

fn compile_shader(gl: SharedContext, src: &str, type_: ShaderType) -> Shader {
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

fn link_program(gl: SharedContext, vertex: &Shader, fragment: &Shader) -> Program {
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
