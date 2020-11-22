#![forbid(missing_debug_implementations)]
#![allow(clippy::new_without_default, clippy::missing_safety_doc)]
#![feature(test, get_mut_unchecked)]

pub mod game_fs;
pub mod gen_idx;
pub mod globals;
pub mod image_decoding_speedrun;
pub mod input;
pub mod utils;

use prelude_plus::*;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::video::{GLProfile, Window};
use sdl2::EventPump;

use cardboard_math::*;
use cardboard_oogl as oogl;

use game_fs::GameFs;
use globals::{Globals, SharedGlobals};

const GAME_NAME: &str = "openKrossKod";
// const GAME_NAME: &str = env!("CARGO_PKG_NAME");
const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");
const GAME_ENGINE_NAME: &str = "Cardboard Engine, \"The Third Impact\" revision";

const GL_CONTEXT_PROFILE: GLProfile = GLProfile::GLES;
const GL_CONTEXT_VERSION: (u8, u8) = (2, 0);

const DEFAULT_WINDOW_SIZE: Vec2<u32> = vec2(568 * 2, 320 * 2);
const BACKGROUND_COLOR: Colorf = colorn(0.1, 1.0);

const RACKET_SIZE: Vec2f = vec2(20.0, 200.0);
const RACKET_OFFSET: f32 = RACKET_SIZE.x;
const RACKET_COLOR: Colorf = colorn(0.9, 1.0);

const RACKET_MAX_SPEED: f32 = 800.0;
const RACKET_ACCELERATION: f32 = 8.0;
const RACKET_SLOWDOWN: f32 = 12.0;
const RACKET_SPEED_EPSILON: f32 = 1.0;

const BALL_SIZE: Vec2f = vec2n(80.0);
const BALL_ROTATION_SPEED: f32 = 1.0;
const BALL_MAX_SPEED: f32 = 1000.0;

fn main() -> AnyResult<()> {
  env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));
  // return image_decoding_speedrun::main();

  info!("{} v{} ({})", GAME_NAME, GAME_VERSION, GAME_ENGINE_NAME);

  let game_fs = GameFs::init().context("Failed to initialize GameFs")?;
  info!("Installation directory: '{}'", game_fs.installation_dir.display());

  let random = globals::GlobalRandom::init()
    .context("Failed to initialize a random number generator (RNG)")?;

  let sdl_context =
    sdl2::init().map_err(AnyError::msg).context("Failed to create an SDL context")?;
  let video_subsystem = sdl_context
    .video()
    .map_err(AnyError::msg)
    .context("Failed to initialize SDL's video subsystem")?;
  let event_pump = sdl_context
    .event_pump()
    .map_err(AnyError::msg)
    .context("Failed to obtain an SDL event pump")?;

  // NOTE: It is very important that GL context attributes are set **before**
  // creating the window! For some reason not doing the initialization in this
  // order on macOS causes ANGLE to not work.
  let gl_attr = video_subsystem.gl_attr();
  gl_attr.set_context_profile(GL_CONTEXT_PROFILE);
  gl_attr.set_context_version(GL_CONTEXT_VERSION.0, GL_CONTEXT_VERSION.1);
  // gl_attr.set_context_flags().debug().set();
  // gl_attr.set_multisample_buffers(1);
  // gl_attr.set_multisample_samples(4);

  let window = video_subsystem
    .window(
      &format!("{} v{}", GAME_NAME, GAME_VERSION),
      DEFAULT_WINDOW_SIZE.x,
      DEFAULT_WINDOW_SIZE.y,
    )
    .resizable()
    .opengl()
    .build()
    .context("Failed to create the game window")?;

  let gl_ctx = window
    .gl_create_context()
    .map_err(AnyError::msg)
    .context("Failed to create an OpenGL context for the game window")?;
  assert_eq!(
    (gl_attr.context_profile(), gl_attr.context_version()),
    (GL_CONTEXT_PROFILE, GL_CONTEXT_VERSION)
  );

  let gl = Rc::new(oogl::Context::load_with(&video_subsystem, gl_ctx));
  debug!("{:?}", gl.capabilities());

  gl.set_blending_factors(oogl::BlendingFactor::SrcAlpha, oogl::BlendingFactor::OneMinusSrcAlpha);
  gl.set_blending_equation(oogl::BlendingEquation::Add);

  let globals = Rc::new({
    let window_size_i = Vec2::from(window.size());
    Globals {
      gl,
      game_fs,
      random,

      time: 0.0,
      delta_time: 0.0,
      fixed_delta_time: 1.0 / 60.0,

      window_size_i,
      window_size: window_size_i.cast_into(),
      window_was_resized: true,

      input_state: input::InputState::new(),
    }
  });

  let mut ball_texture = oogl::Texture2D::new(Rc::clone(&globals.gl));
  {
    let bound_texture = ball_texture.bind(None);
    bound_texture.set_wrapping_modes(oogl::TextureWrappingMode::Repeat);
    bound_texture.set_filters(oogl::TextureFilter::Linear, None);
    load_texture_data_from_png(0, &bound_texture, globals.game_fs.open_file("trololo.png")?)
      .context("Failed to decode 'trololo.png'")?;
  }

  let renderer =
    Renderer::init(Rc::clone(&globals)).context("Failed to initialize the renderer")?;

  let mut game = Game {
    should_stop: false,
    globals,
    state: GameState {
      left_racket: Racket { pos: 0.0, vel: 0.0, accel: 0.0 },
      right_racket: Racket { pos: 0.0, vel: 0.0, accel: 0.0 },
      ball: Ball {
        pos: vec2n(0.0),
        vel: vec2::<f32>(0.8, 0.5).normalize() * BALL_MAX_SPEED,
        rotation: 0.0,
        rotation_speed: BALL_ROTATION_SPEED,
      },
    },

    sdl_context,
    video_subsystem,
    window,
    event_pump,
    renderer,
    ball_texture,
  };

  game.start_loop()
}

#[derive(Debug)]
struct GameState {
  pub left_racket: Racket,
  pub right_racket: Racket,
  pub ball: Ball,
}

#[derive(Debug)]
struct Racket {
  pub pos: f32,
  pub vel: f32,
  pub accel: f32,
}

#[derive(Debug)]
struct Ball {
  pub pos: Vec2f,
  pub vel: Vec2f,
  pub rotation: f32,
  pub rotation_speed: f32,
}

struct Game {
  pub should_stop: bool,
  pub globals: SharedGlobals,
  pub state: GameState,

  pub sdl_context: sdl2::Sdl,
  pub video_subsystem: sdl2::VideoSubsystem,
  pub window: Window,
  pub event_pump: EventPump,
  pub renderer: Renderer,
  pub ball_texture: oogl::Texture2D,
}

impl Game {
  pub fn start_loop(&mut self) -> AnyResult<()> {
    let mut prev_time = Instant::now();
    let mut fixed_update_time_accumulator = 0.0;

    self.globals.gl.clear_color(BACKGROUND_COLOR);
    self.window.gl_swap_window();

    while !self.should_stop {
      let current_time = Instant::now();
      let delta_time = (current_time - prev_time).as_secs_f64();
      unsafe { Rc::get_mut_unchecked(&mut self.globals) }.delta_time = delta_time;

      self.process_input();

      fixed_update_time_accumulator += delta_time;
      let fixed_delta_time = self.globals.fixed_delta_time;
      // FIXME: What if a lot of time has passed between frames, e.g. due to the
      // game being suspended or paused with SIGSTOP (and resumed with SIGCONT)?
      while fixed_update_time_accumulator >= fixed_delta_time {
        self.fixed_update();
        fixed_update_time_accumulator -= fixed_delta_time;
        unsafe { Rc::get_mut_unchecked(&mut self.globals) }.time += fixed_delta_time;
      }

      self.update();

      if cfg!(feature = "gl_debug_all_commands") {
        println!("================ [OpenGL] ================");
      }

      self.globals.gl.clear_color(BACKGROUND_COLOR);
      self.render();
      self.window.gl_swap_window();

      prev_time = current_time;
    }

    Ok(())
  }

  pub fn process_input(&mut self) {
    let globals = unsafe { Rc::get_mut_unchecked(&mut self.globals) };
    let main_window_id = self.window.id();

    globals.window_was_resized = false;

    for event in self.event_pump.poll_iter() {
      match event {
        Event::Quit { .. } | Event::AppTerminating { .. } | Event::AppLowMemory { .. } => {
          self.should_stop = true;
        }

        Event::Window { window_id, win_event: WindowEvent::Close, .. }
          if window_id == main_window_id =>
        {
          self.should_stop = true;
        }

        Event::Window { window_id, win_event: WindowEvent::SizeChanged(w, h), .. }
          if window_id == main_window_id =>
        {
          assert!(w > 0);
          assert!(h > 0);
          globals.window_size_i = vec2(w as u32, h as u32);
          globals.window_size = vec2(w as f32, h as f32);
          globals.window_was_resized = true;
        }

        Event::MouseMotion { window_id, x, y, .. } if window_id == main_window_id => {
          globals.input_state.mouse_pos =
            vec2(x as f32 - globals.window_size.x * 0.5, globals.window_size.y * 0.5 - y as f32);
        }

        Event::KeyDown { window_id, repeat: false, scancode: Some(scancode), .. }
          if window_id == main_window_id =>
        {
          globals.input_state.keyboard.set(scancode, true);
        }

        Event::KeyUp { window_id, repeat: false, scancode: Some(scancode), .. }
          if window_id == main_window_id =>
        {
          globals.input_state.keyboard.set(scancode, false);
        }

        _ => {}
      }
    }
  }

  pub fn update(&mut self) {
    if self.globals.input_state.is_key_down(Scancode::Q) {
      self.should_stop = true;
    }
  }

  pub fn fixed_update(&mut self) {
    let fixed_delta_time = self.globals.fixed_delta_time as f32;
    let window_size = self.globals.window_size;

    for (racket, key_up, key_down) in &mut [
      (&mut self.state.left_racket, Scancode::W, Scancode::S),
      (&mut self.state.right_racket, Scancode::Up, Scancode::Down),
    ] {
      let mut dir = 0.0;
      if self.globals.input_state.is_key_down(*key_up) {
        dir += 1.0;
      }
      if self.globals.input_state.is_key_down(*key_down) {
        dir -= 1.0;
      }
      racket.accel = dir * RACKET_MAX_SPEED * RACKET_ACCELERATION;

      racket.vel +=
        if dir != 0.0 { racket.accel } else { -racket.vel * RACKET_SLOWDOWN } * fixed_delta_time;
      racket.vel = racket.vel.clamp2_abs(RACKET_MAX_SPEED);
      if racket.vel.abs() < RACKET_SPEED_EPSILON {
        racket.vel = 0.0;
      }

      racket.pos += racket.vel * fixed_delta_time;
      racket.pos = racket.pos.clamp2_abs((window_size.y / 2.0 - RACKET_SIZE.y / 2.0).abs());
    }

    {
      let ball = &mut self.state.ball;
      ball.pos += ball.vel * fixed_delta_time;

      let bouncing_bounds: Vec2f = window_size / 2.0 - BALL_SIZE / 2.0;
      if ball.pos.x.abs() >= bouncing_bounds.x {
        ball.vel.x = -ball.vel.x;
      }
      if ball.pos.y.abs() >= bouncing_bounds.y {
        ball.vel.y = -ball.vel.y;
      }

      ball.pos = ball.pos.clamp2_abs(bouncing_bounds.abs());

      ball.rotation += ball.rotation_speed * f32::consts::TAU * fixed_delta_time;
    }
  }

  pub fn render(&mut self) {
    self.globals.gl.set_viewport(vec2n(0), self.globals.window_size_i.cast_into());

    self.renderer.prepare();

    let window_size = self.globals.window_size;
    for (side, racket) in &[(-1.0, &self.state.left_racket), (1.0, &self.state.right_racket)] {
      self.renderer.draw_shape(Shape {
        type_: ShapeType::Rectangle,
        pos: vec2(
          side * (window_size.x / 2.0 - RACKET_SIZE.x / 2.0 - 2.0 * RACKET_OFFSET),
          racket.pos,
        ),
        size: RACKET_SIZE,
        rotation: 0.0,
        fill: ShapeFill::Color(RACKET_COLOR),
      });
    }

    self.renderer.draw_shape(Shape {
      type_: ShapeType::Ellipse,
      pos: self.state.ball.pos,
      size: BALL_SIZE,
      rotation: self.state.ball.rotation,
      fill: ShapeFill::Texture(self.ball_texture.bind(None)),
    });
  }
}

struct Renderer {
  globals: SharedGlobals,

  vbo: oogl::VertexBuffer<[i8; 2]>,
  white_texture: oogl::Texture2D,

  rectangle_program: oogl::Program,
  rectangle_uniform_window_size: oogl::Uniform<Vec2f>,
  rectangle_uniform_pos: oogl::Uniform<Vec2f>,
  rectangle_uniform_size: oogl::Uniform<Vec2f>,
  rectangle_uniform_rotation: oogl::Uniform<f32>,
  rectangle_uniform_color: oogl::Uniform<Colorf>,
  rectangle_uniform_tex: oogl::Uniform<u32>,

  ellipse_program: oogl::Program,
  ellipse_uniform_window_size: oogl::Uniform<Vec2f>,
  ellipse_uniform_pos: oogl::Uniform<Vec2f>,
  ellipse_uniform_size: oogl::Uniform<Vec2f>,
  ellipse_uniform_rotation: oogl::Uniform<f32>,
  ellipse_uniform_color: oogl::Uniform<Colorf>,
  ellipse_uniform_tex: oogl::Uniform<u32>,
}

impl Renderer {
  fn init(globals: SharedGlobals) -> AnyResult<Self> {
    let gl = &globals.gl;

    let common_vertex_shader = compile_shader(
      Rc::clone(&gl),
      &globals.game_fs.read_binary_file("shaders/shape.vert.glsl")?,
      oogl::ShaderType::Vertex,
    )
    .context("Failed to link the 'shaders/shape.vert.glsl'")?;

    let rectangle_fragment_shader = compile_shader(
      Rc::clone(&gl),
      &globals.game_fs.read_binary_file("shaders/rectangle.frag.glsl")?,
      oogl::ShaderType::Fragment,
    )
    .context("Failed to link the 'shaders/rectangle.frag.glsl'")?;

    let ellipse_fragment_shader = compile_shader(
      Rc::clone(&gl),
      &globals.game_fs.read_binary_file("shaders/ellipse.frag.glsl")?,
      oogl::ShaderType::Fragment,
    )
    .context("Failed to link the 'shaders/ellipse.frag.glsl'")?;

    let rectangle_program =
      link_program(Rc::clone(&gl), &[&common_vertex_shader, &rectangle_fragment_shader])
        .context("Failed to link the rectangle program")?;
    let rectangle_attribute_pos = rectangle_program.get_attribute(b"a_pos");
    let rectangle_uniform_window_size = rectangle_program.get_uniform(b"u_window_size");
    let rectangle_uniform_pos = rectangle_program.get_uniform(b"u_pos");
    let rectangle_uniform_size = rectangle_program.get_uniform(b"u_size");
    let rectangle_uniform_rotation = rectangle_program.get_uniform(b"u_rotation");
    let rectangle_uniform_color = rectangle_program.get_uniform(b"u_color");
    let rectangle_uniform_tex = rectangle_program.get_uniform(b"u_tex");

    let ellipse_program =
      link_program(Rc::clone(&gl), &[&common_vertex_shader, &ellipse_fragment_shader])
        .context("Failed to link the ellipse program")?;
    let ellipse_attribute_pos = ellipse_program.get_attribute(b"a_pos");
    let ellipse_uniform_window_size = ellipse_program.get_uniform(b"u_window_size");
    let ellipse_uniform_pos = ellipse_program.get_uniform(b"u_pos");
    let ellipse_uniform_size = ellipse_program.get_uniform(b"u_size");
    let ellipse_uniform_rotation = ellipse_program.get_uniform(b"u_rotation");
    let ellipse_uniform_color = ellipse_program.get_uniform(b"u_color");
    let ellipse_uniform_tex = ellipse_program.get_uniform(b"u_tex");

    assert_eq!(rectangle_attribute_pos.location(), ellipse_attribute_pos.location());
    assert_eq!(rectangle_attribute_pos.data_type(), ellipse_attribute_pos.data_type());

    let mut vbo = oogl::VertexBuffer::new(
      Rc::clone(&gl),
      // this attribute pointer will be the same for both programs because both
      // use the same vertex shader, as such the VBO can be shared
      vec![rectangle_attribute_pos.to_pointer(oogl::AttributePtrConfig {
        type_: oogl::AttributePtrDataType::I8,
        len: 2,
        normalize: false,
      })],
    );

    {
      let bound_vbo = vbo.bind();
      bound_vbo.enable_attributes();
      bound_vbo.configure_attributes();
      bound_vbo.set_data(&[[-1, -1], [-1, 1], [1, 1], [1, -1]], oogl::BufferUsageHint::StaticDraw);
    }

    let mut white_texture = oogl::Texture2D::new(Rc::clone(&gl));
    {
      let bound_texture = white_texture.bind(None);
      bound_texture.set_wrapping_modes(oogl::TextureWrappingMode::Repeat);
      bound_texture.set_filters(oogl::TextureFilter::Linear, None);
      bound_texture.set_data(
        0,
        oogl::TextureInputFormat::Luminance,
        oogl::TextureInternalFormat::Luminance,
        vec2n(1),
        &[255],
      )
    }

    Ok(Self {
      globals,

      vbo,
      white_texture,

      rectangle_program,
      rectangle_uniform_window_size,
      rectangle_uniform_pos,
      rectangle_uniform_size,
      rectangle_uniform_rotation,
      rectangle_uniform_color,
      rectangle_uniform_tex,

      ellipse_program,
      ellipse_uniform_window_size,
      ellipse_uniform_pos,
      ellipse_uniform_size,
      ellipse_uniform_rotation,
      ellipse_uniform_color,
      ellipse_uniform_tex,
    })
  }

  fn prepare(&mut self) {
    let window_size = self.globals.window_size;

    {
      let program = self.rectangle_program.bind();
      self.rectangle_uniform_window_size.set(&program, window_size);
    }
    {
      let program = self.ellipse_program.bind();
      self.ellipse_uniform_window_size.set(&program, window_size);
    }
  }

  fn draw_shape(&mut self, shape: Shape<'_>) {
    let (color, bound_texture) = match shape.fill {
      ShapeFill::Color(color) => (color, self.white_texture.bind(None)),
      ShapeFill::Texture(bound_texture) => (colorn(1.0, 1.0), bound_texture),
    };

    let program = match shape.type_ {
      ShapeType::Rectangle => {
        let program = self.rectangle_program.bind();
        self.rectangle_uniform_pos.set(&program, shape.pos);
        self.rectangle_uniform_size.set(&program, shape.size);
        self.rectangle_uniform_rotation.set(&program, shape.rotation);
        self.rectangle_uniform_color.set(&program, color);
        self.rectangle_uniform_tex.set(&program, bound_texture.unit());
        program
      }
      ShapeType::Ellipse => {
        let program = self.ellipse_program.bind();
        self.ellipse_uniform_pos.set(&program, shape.pos);
        self.ellipse_uniform_size.set(&program, shape.size);
        self.ellipse_uniform_rotation.set(&program, shape.rotation);
        self.ellipse_uniform_color.set(&program, color);
        self.ellipse_uniform_tex.set(&program, bound_texture.unit());
        program
      }
    };

    let bound_vbo = self.vbo.bind();
    bound_vbo.draw(&program, oogl::DrawPrimitive::TriangleFan, 0, 4);
  }
}

#[derive(Debug)]
struct Shape<'a> {
  type_: ShapeType,
  pos: Vec2f,
  size: Vec2f,
  rotation: f32,
  fill: ShapeFill<'a>,
}

#[derive(Debug)]
enum ShapeType {
  Rectangle,
  Ellipse,
}

#[derive(Debug)]
enum ShapeFill<'a> {
  Color(Colorf),
  Texture(oogl::Texture2DBinding<'a>),
}

// fn create_program(
//   gl: oogl::SharedContext,
//   vertex_shader_src: &[u8],
//   fragment_shader_src: &[u8],
// ) -> oogl::Program {
//   let vs = compile_shader(Rc::clone(&gl), vertex_shader_src, oogl::ShaderType::Vertex);
//   let fs = compile_shader(Rc::clone(&gl), fragment_shader_src, oogl::ShaderType::Fragment);
//   link_program(gl, &[&vs, &fs])
// }

fn compile_shader(
  ctx: oogl::SharedContext,
  src: &[u8],
  type_: oogl::ShaderType,
) -> AnyResult<oogl::Shader> {
  let shader = oogl::Shader::new(ctx, type_);
  shader.set_source(src);

  let success = shader.compile();
  let log = shader.get_info_log();
  let log = String::from_utf8_lossy(&log);
  if !success {
    bail!("Shader compilation error(s):\n{}", log);
  } else if !log.is_empty() {
    warn!("Shader compilation warning(s):\n{}", log);
  }

  Ok(shader)
}

fn link_program(ctx: oogl::SharedContext, shaders: &[&oogl::Shader]) -> AnyResult<oogl::Program> {
  let program = oogl::Program::new(ctx);
  for shader in shaders {
    program.attach_shader(shader);
  }

  let success = program.link();
  let log = program.get_info_log();
  let log = String::from_utf8_lossy(&log);
  if !success {
    bail!("Program linking error: {}", log);
  } else if !log.is_empty() {
    warn!("Program linking warning: {}", log);
  }

  for shader in shaders {
    program.detach_shader(shader);
  }
  Ok(program)
}

fn load_texture_data_from_png<R: Read>(
  level_of_detail: u32,
  bound_texture: &oogl::Texture2DBinding<'_>,
  reader: R,
) -> Result<Vec2<u32>, png::DecodingError> {
  let decoder = png::Decoder::new(reader);
  let (info, mut reader) = decoder.read_info()?;
  let mut buf = vec![0; info.buffer_size()];
  reader.next_frame(&mut buf)?;

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

  bound_texture.set_data(
    level_of_detail,
    gl_format,
    gl_internal_format,
    vec2(info.width, info.height),
    &buf,
  );

  Ok(vec2(info.width, info.height))
}
