/*!

_**NOTE(2020-11-22)** from my future self:_

Greetings, Seeker! I'm glad you took interest in the early history of the
openKrossKod codebase and internals. Here's the source code of the famous
(well, among certain experts) "Spinning San-Cheese" demo, along with the early
takes on the modules which will later be called `cardboard_oogl` and
`cardboard_math`. [Here it is in
action](https://cdn.discordapp.com/attachments/382339402338402317/757002993093574717/simplescreenrecorder-2020-09-20_01.11.50_enc.mp4)
([the original, not re-encoded
file](https://cdn.discordapp.com/attachments/701049519701491712/757001316664082432/simplescreenrecorder-2020-09-20_01.11.50.mp4)).

*/

#![allow(clippy::new_without_default)]
#![feature(negative_impls)]
#![feature(debug_non_exhaustive)]
#![feature(const_fn)]

pub mod gl;
pub mod gl_prelude;
pub mod math;
pub mod oogl;
pub mod prelude;
pub mod utils;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::video::GLProfile;

use gl_prelude::*;
use math::ops::{Lerp, RangeMap};
use math::{vec2, vec2n, Vec2, Vec2d, Vec2f};
use oogl::{BoundBuffer, BoundTexture, SetUniform};
use prelude::*;
use rand::{thread_rng, Rng};

const GL_CONTEXT_PROFILE: GLProfile = GLProfile::GLES;
const GL_CONTEXT_VERSION: (u8, u8) = (2, 0);

const FLOATS_PER_VERTEX: usize = 2 + 3 + 2 + 1;
const VERTICES_COUNT: usize = 3;

#[rustfmt::skip]
const VERTEX_DATA: [GLfloat; FLOATS_PER_VERTEX * VERTICES_COUNT] = [
  // x     y     r    g    b     t    u    i
    0.5, -0.5,  1.0, 0.0, 0.0,  0.0, 0.0, 0.5,
   -0.5, -0.5,  0.0, 1.0, 0.0,  1.0, 0.0, 0.5,
    0.0,  0.5,  0.0, 0.0, 1.0,  0.5, 1.0, 0.5,
];
const ELEMENT_DATA: [GLushort; VERTICES_COUNT] = [0, 1, 2];

// const VERTEX_DATA: [GLfloat; 8] = [-1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0];
// const ELEMENT_DATA: [GLushort; 6] = [0, 1, 2, 2, 3, 0];

const VS_SRC: &[u8] = include_bytes!("shaders/color.vert");
const FS_SRC: &[u8] = include_bytes!("shaders/color.frag");
const IMAGE_DATA: &[u8] = include_bytes!("../SanCheese.png");

fn compile_shader(ctx: Rc<oogl::Context>, src: &[u8], type_: oogl::ShaderType) -> oogl::Shader {
  let shader = oogl::Shader::new(ctx, type_);
  shader.set_source(src);

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

fn link_program(ctx: Rc<oogl::Context>, shaders: &[&oogl::Shader]) -> oogl::ShaderProgram {
  let program = oogl::ShaderProgram::new(ctx);
  for shader in shaders {
    program.attach_shader(shader);
  }

  let success = program.link();
  let log = program.get_info_log();
  let log = String::from_utf8_lossy(&log);
  if !success {
    panic!("Program linking error: {}", log);
  } else if !log.is_empty() {
    eprintln!("Program linking warning: {}", log);
  }

  for shader in shaders {
    program.detach_shader(shader);
  }
  program
}

fn main() {
  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();

  let window_title = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));
  let window =
    video_subsystem.window(window_title, 800, 600).resizable().opengl().build().unwrap();

  let gl_attr = video_subsystem.gl_attr();
  gl_attr.set_context_profile(GL_CONTEXT_PROFILE);
  gl_attr.set_context_version(GL_CONTEXT_VERSION.0, GL_CONTEXT_VERSION.1);
  gl_attr.set_context_flags().debug().set();

  let gl_ctx = window.gl_create_context().unwrap();
  assert_eq!(gl_attr.context_profile(), GL_CONTEXT_PROFILE);
  assert_eq!(gl_attr.context_version(), GL_CONTEXT_VERSION);

  let gl = Rc::new(oogl::Context::load_with(|name| {
    video_subsystem.gl_get_proc_address(name) as *const _
  }));

  let mut event_pump = sdl_context.event_pump().unwrap();

  // Create GLSL shaders
  let vs = compile_shader(Rc::clone(&gl), VS_SRC, oogl::ShaderType::Vertex);
  let fs = compile_shader(Rc::clone(&gl), FS_SRC, oogl::ShaderType::Fragment);
  let mut program = link_program(Rc::clone(&gl), &[&vs, &fs]);

  let uniform_tex_size = program.get_uniform(b"tex_size");
  let uniform_window_size = program.get_uniform(b"window_size");
  let uniform_time = program.get_uniform(b"time");
  let uniform_tex = program.get_uniform(b"tex");
  let uniform_random_seed = program.get_uniform(b"random_seed");
  let uniform_random = program.get_uniform(b"random");

  let vertex_attr_pos = program.get_attribute_location(b"position");
  let vertex_attr_color = program.get_attribute_location(b"color");
  let vertex_attr_texcoord = program.get_attribute_location(b"texcoord");
  let vertex_attr_color_intensity = program.get_attribute_location(b"color_intensity");

  let program_bound = program.bind();

  let texture_unit = 0;
  let mut texture = oogl::Texture::new(gl.clone());
  let texture_bound = oogl::BoundTexture2D::new(&mut texture, Some(texture_unit));
  texture_bound.set_wrapping_modes(oogl::TextureWrappingMode::ClampToEdge);
  texture_bound.set_filters(oogl::TextureFilter::Nearest, None);
  if let Some(tex_uniform) = &uniform_tex {
    tex_uniform.set(&gl, texture_unit as u32);
  }

  {
    // let decoder = png::Decoder::new(File::open("SanCheese2.png").unwrap());
    let decoder = png::Decoder::new(IMAGE_DATA);
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();

    use oogl::{TextureInputFormat, TextureInternalFormat};
    use png::{BitDepth, ColorType};

    match info.bit_depth {
      BitDepth::Eight => {}
      _ => unimplemented!("Unsupported texture bit depth: {:?}", info.bit_depth),
    }

    let (gl_format, gl_internal_format) = match info.color_type {
      ColorType::Grayscale => (TextureInputFormat::Luminance, TextureInternalFormat::Luminance),
      ColorType::RGB => (TextureInputFormat::RGB, TextureInternalFormat::RGB),
      ColorType::GrayscaleAlpha => {
        (TextureInputFormat::LuminanceAlpha, TextureInternalFormat::LuminanceAlpha)
      }
      ColorType::RGBA => (TextureInputFormat::RGBA, TextureInternalFormat::RGBA),
      _ => unimplemented!("Unsupported texture color type: {:?}", info.color_type),
    };

    texture_bound.set_data(0, gl_format, gl_internal_format, (info.width, info.height), &buf);

    if let Some(tex_size_uniform) = &uniform_tex_size {
      tex_size_uniform.set(&gl, (info.width as f32, info.height as f32));
    }
  }

  let mut vbo = oogl::Buffer::new(gl.clone());
  let vbo_bound = oogl::BoundVertexBuffer::new(&mut vbo);
  // vbo_bound.set_data(&VERTEX_DATA, oogl::BufferUsageHint::StaticDraw);

  #[allow(clippy::erasing_op)]
  unsafe {
    let float_size = mem::size_of::<GLfloat>();
    let stride = (FLOATS_PER_VERTEX * float_size) as GLsizei;

    if let Some(vertex_attr_pos) = vertex_attr_pos {
      gl.gl.EnableVertexAttribArray(vertex_attr_pos);
      gl.gl.VertexAttribPointer(
        vertex_attr_pos,
        2,
        gl::FLOAT,
        gl::FALSE,
        stride,
        (0 * float_size) as *const GLvoid,
      );
    }

    if let Some(vertex_attr_color) = vertex_attr_color {
      gl.gl.EnableVertexAttribArray(vertex_attr_color);
      gl.gl.VertexAttribPointer(
        vertex_attr_color,
        3,
        gl::FLOAT,
        gl::FALSE,
        stride,
        (2 * float_size) as *const GLvoid,
      );
    }

    if let Some(vertex_attr_texcoord) = vertex_attr_texcoord {
      gl.gl.EnableVertexAttribArray(vertex_attr_texcoord);
      gl.gl.VertexAttribPointer(
        vertex_attr_texcoord,
        2,
        gl::FLOAT,
        gl::FALSE,
        stride,
        (5 * float_size) as *const GLvoid,
      );
    }

    if let Some(vertex_attr_color_intensity) = vertex_attr_color_intensity {
      gl.gl.EnableVertexAttribArray(vertex_attr_color_intensity);
      gl.gl.VertexAttribPointer(
        vertex_attr_color_intensity,
        1,
        gl::FLOAT,
        gl::FALSE,
        stride,
        (7 * float_size) as *const GLvoid,
      );
    }
  }

  let mut ebo = oogl::Buffer::new(gl.clone());
  let ebo_bound = oogl::BoundElementBuffer::new(&mut ebo);
  ebo_bound.set_data(&ELEMENT_DATA, oogl::BufferUsageHint::StaticDraw);

  let mut time: f64 = 0.0;

  let mut rng = thread_rng();
  if let Some(random_seed_uniform) = &uniform_random_seed {
    random_seed_uniform.set(&gl, rng.gen::<f32>());
  }

  let mut vertices: [Vec2f; VERTICES_COUNT] = [vec2(0.5, -0.5), vec2(-0.5, -0.5), vec2(0.0, 0.5)];
  let texcoords: [Vec2f; VERTICES_COUNT] = [vec2(0.0, 1.0), vec2(1.0, 1.0), vec2(0.5, 0.0)];
  let mut selected_vertex = None::<usize>;

  let mut window_size: Vec2<u32> = Vec2::from(window.size());
  let mut mouse_pos: Vec2d = vec2n(0.0);

  let mut prev_time = Instant::now();
  'game_loop: loop {
    let current_time = Instant::now();
    let delta_time: f64 = (current_time - prev_time).as_secs_f64();
    prev_time = current_time;

    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } | Event::KeyUp { keycode: Some(Keycode::Escape), .. } => {
          break 'game_loop;
        }

        Event::Window { win_event: WindowEvent::SizeChanged(w, h), .. } => {
          assert!(w > 0 && h > 0, "w = {}, h = {}", w, h);
          window_size = vec2(w as u32, h as u32);
          gl.set_viewport(0, 0, w, h);
        }

        Event::MouseMotion { x, y, .. } => {
          mouse_pos = window_coords_to_gl_coords(vec2(
            x as f64 / window_size.x as f64,
            y as f64 / window_size.y as f64,
          ));
        }

        Event::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
          let mut min_distance = f64::INFINITY;

          for i in 0..VERTICES_COUNT {
            let distance = mouse_pos.sqr_distance(vertices[i].as_f64());
            if distance < min_distance {
              min_distance = distance;
              selected_vertex = Some(i);
            }
          }
        }

        Event::MouseButtonUp { mouse_btn: MouseButton::Left, .. } => {
          selected_vertex = None;
        }

        _ => {}
      }
    }

    gl.clear_color(0.3, 0.3, 0.3, 1.0);

    time += delta_time;
    if let Some(time_uniform) = &uniform_time {
      time_uniform.set(&gl, time as f32);
    }

    if let Some(random_uniform) = &uniform_random {
      random_uniform.set(&gl, rng.gen::<f32>());
    }

    if let Some(window_size_uniform) = &uniform_window_size {
      window_size_uniform.set(&gl, (window_size.x as f32, window_size.y as f32));
    }

    let mut vertex_data = VERTEX_DATA;
    fn set_vertex_data_float(
      vertex_data: &mut [f32],
      vertex_index: usize,
      field_offset: usize,
      value: f32,
    ) {
      vertex_data[vertex_index * FLOATS_PER_VERTEX + field_offset] = value;
    }
    fn set_vertex_data_vec2(
      vertex_data: &mut [f32],
      vertex_index: usize,
      field_offset: usize,
      value: Vec2f,
    ) {
      set_vertex_data_float(vertex_data, vertex_index, field_offset, value.x);
      set_vertex_data_float(vertex_data, vertex_index, field_offset + 1, value.y);
    }

    if let Some(selected_vertex) = selected_vertex {
      vertices[selected_vertex] = mouse_pos.as_f32();
    }

    if let Some(random_uniform) = &uniform_random {
      random_uniform.set(&gl, rng.gen::<f32>());
    }

    let texcoords_rotation = (time % VERTICES_COUNT as f64).floor() as usize;
    for i in 0..VERTICES_COUNT {
      let vertex = vertices[i];
      set_vertex_data_vec2(&mut vertex_data, i, 0, vertex);

      let current_texcoord = texcoords[(texcoords_rotation + i) % VERTICES_COUNT];
      let next_texcoord = texcoords[(texcoords_rotation + i + 1) % VERTICES_COUNT];
      set_vertex_data_vec2(
        &mut vertex_data,
        i,
        5,
        current_texcoord.lerp(next_texcoord, time.fract() as f32),
      );

      let intensity_rotation = 2.0 * i as f64 / VERTICES_COUNT as f64;
      set_vertex_data_float(
        &mut vertex_data,
        i,
        7,
        (((f64::consts::PI * (-time + intensity_rotation)).sin() + 1.0) / 2.0)
          .range_map((0.0, 1.0), (1.0 / 8.0, 2.0 / 3.0)) as f32,
      );
    }

    vbo_bound.set_data(&vertex_data, oogl::BufferUsageHint::DynamicDraw);

    ebo_bound.draw(&program_bound, oogl::DrawPrimitive::Triangles, 0, VERTICES_COUNT as u32);
    // vbo_bound.draw(&program_bound, oogl::DrawPrimitive::TriangleFan, 0, VERTICES_COUNT as u32);

    window.gl_swap_window();

    thread::sleep(time::Duration::new(0, 1_000_000_000u32 / 60));
  }

  drop(gl_ctx);
}

fn window_coords_to_gl_coords(point: Vec2d) -> Vec2d {
  vec2(point.x * 2.0 - 1.0, 1.0 - point.y * 2.0)
}
