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
  texcoord: Vec2f,
}

#[rustfmt::skip]
const VERTEX_DATA: &[Vertex] = &[
  Vertex { pos: vec2( 0.5,  0.5), color: color(1.0, 0.6, 0.6, 1.0), texcoord: vec2(1.0, 0.0) },
  Vertex { pos: vec2( 0.5, -0.5), color: color(1.0, 0.6, 0.6, 1.0), texcoord: vec2(1.0, 1.0) },
  Vertex { pos: vec2(-0.5,  0.5), color: color(0.6, 1.0, 0.6, 1.0), texcoord: vec2(0.0, 0.0) },
  Vertex { pos: vec2(-0.5, -0.5), color: color(0.6, 0.6, 1.0, 1.0), texcoord: vec2(0.0, 1.0) },
];

static VS_SRC: &str = r#"#version 100
  attribute vec2 a_pos;
  attribute vec4 a_color;
  attribute vec2 a_texcoord;
  varying   vec4 v_color;
  varying   vec2 v_texcoord;
  void main(void) {
    gl_Position = vec4(a_pos, 0.0, 1.0);
    v_color = a_color;
    v_texcoord = a_texcoord;
  }
"#;

static FS_SRC: &str = r#"#version 100
  precision highp float;
  varying vec4 v_color;
  varying vec2 v_texcoord;
  uniform sampler2D u_tex1;
  uniform sampler2D u_tex2;
  void main() {
    gl_FragColor = mix(texture2D(u_tex1, v_texcoord), texture2D(u_tex2, v_texcoord), 0.2) * v_color;
  }
"#;

const IMAGE_DATA_1: &[u8] = include_bytes!("./assets/LearnOpenGL/container.jpeg");
const IMAGE_DATA_2: &[u8] = include_bytes!("./assets/LearnOpenGL/awesomeface.png");

fn main() {
  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();

  let gl_attr = video_subsystem.gl_attr();
  gl_attr.set_context_profile(GLProfile::GLES);
  gl_attr.set_context_version(2, 0);

  let window = video_subsystem
    .window("cardboard_oogl textures example", 800, 600)
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
  let attr_texcoord = program.get_attrib::<Vec2f>("a_texcoord");
  let uni_tex1 = program.get_uniform::<TextureUnit>("u_tex1");
  let uni_tex2 = program.get_uniform::<TextureUnit>("u_tex2");

  let bound_program = program.bind();

  let mut vbo = VertexBuffer::<Vertex>::new(
    gl.share(),
    vec![
      attr_pos.to_pointer_simple(),
      attr_color.to_pointer_simple(),
      attr_texcoord.to_pointer_simple(),
    ],
  );

  let bound_vbo = vbo.bind();
  bound_vbo.enable_attribs();
  bound_vbo.configure_attribs();
  bound_vbo.reserve_and_set(BufferUsageHint::StaticDraw, &VERTEX_DATA);

  let texture_unit1 = TextureUnit::new(gl.share());
  let texture_unit2 = TextureUnit::new(gl.share());
  uni_tex1.set(&bound_program, &texture_unit1);
  uni_tex2.set(&bound_program, &texture_unit2);

  let mut texture1 = load_jpeg_texture_2d(gl.share(), IMAGE_DATA_1);
  {
    let bound_texture1 = texture1.bind(&texture_unit1);
    bound_texture1.set_wrapping_modes(TextureWrappingMode::Repeat);
    bound_texture1.set_filters(TextureFilter::Linear, Some(TextureFilter::Linear));
    bound_texture1.generate_mipmap();
  }

  let mut texture2 = load_png_texture_2d(gl.share(), IMAGE_DATA_2);
  {
    let bound_texture2 = texture2.bind(&texture_unit2);
    bound_texture2.set_wrapping_modes(TextureWrappingMode::Repeat);
    bound_texture2.set_filters(TextureFilter::Linear, Some(TextureFilter::Linear));
    bound_texture2.generate_mipmap();
  }

  gl.set_clear_color(colorn(0.1, 1.0));

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
    bound_vbo.draw(&bound_program, DrawPrimitive::TriangleStrip);

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

fn load_png_texture_2d(gl: SharedContext, encoded_data: &[u8]) -> Texture2D {
  let decoder = png::Decoder::new(encoded_data);
  let (info, mut reader) = decoder.read_info().unwrap();
  let mut buf = vec![0; info.buffer_size()];
  reader.next_frame(&mut buf).unwrap();

  assert!(info.bit_depth == png::BitDepth::Eight);
  assert!(info.color_type == png::ColorType::RGBA);

  let texture_unit = TextureUnit::new(gl.share());
  let mut texture = Texture2D::new(gl, TextureInputFormat::RGBA, None);
  {
    let bound_texture = texture.bind(&texture_unit);
    bound_texture.set_size(vec2(info.width, info.height));
    bound_texture.reserve_and_set(0, &buf);
  }

  texture
}

fn load_jpeg_texture_2d(gl: SharedContext, encoded_data: &[u8]) -> Texture2D {
  let mut decoder = jpeg_decoder::Decoder::new(encoded_data);
  let buf = decoder.decode().unwrap();
  let info = decoder.info().unwrap();

  assert!(info.pixel_format == jpeg_decoder::PixelFormat::RGB24);

  let texture_unit = TextureUnit::new(gl.share());
  let mut texture = Texture2D::new(gl, TextureInputFormat::RGB, None);
  {
    let bound_texture = texture.bind(&texture_unit);
    bound_texture.set_size(vec2(info.width as u32, info.height as u32));
    bound_texture.reserve_and_set(0, &buf);
  }

  texture
}
