#![deny(missing_debug_implementations)]
#![allow(clippy::new_without_default, clippy::missing_safety_doc)]
#![feature(test, get_mut_unchecked)]

pub mod gen_idx; // TODO

pub mod game_fs;
pub mod globals;
pub mod input;
pub mod profiling;
pub mod renderer;

pub mod game_of_life;
pub mod image_decoding_speedrun;
pub mod mandelbrot;
pub mod marching_squares;
pub mod pong;

use prelude_plus::*;
use sdl2::event::{Event, WindowEvent};
use sdl2::video::{GLProfile, Window};
use sdl2::EventPump;

use cardboard_math::*;
use cardboard_oogl as oogl;

use crate::game_fs::*;
use crate::globals::*;
use crate::input::Key;
use crate::renderer::*;

use crate::pong::Pong;

#[cfg(feature = "game_of_life")]
use crate::game_of_life::GameOfLife;
#[cfg(feature = "mandelbrot")]
use crate::mandelbrot::Mandelbrot;
#[cfg(feature = "marching_squares")]
use crate::marching_squares::MarchingSquares;

const GAME_NAME: &str = "openKrossKod";
// const GAME_NAME: &str = env!("CARGO_PKG_NAME");
const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");
const GAME_ENGINE_NAME: &str = "Cardboard Engine, \"The Third Impact\" revision";

const GL_CONTEXT_PROFILE: GLProfile = GLProfile::GLES;
const GL_CONTEXT_VERSION: (u8, u8) = (2, 0);

const DEFAULT_WINDOW_SIZE: Vec2u32 = vec2(568 * 2, 320 * 2);
const BACKGROUND_COLOR: Colorf = colorn(0.1, 1.0);

const GAME_LOOP_IDLING_WAIT_INTERVAL: f64 = 1.0 / 20.0;

fn main() {
  if let Err(err) = try_main() {
    if log_enabled!(LogLevel::Error) {
      error!("{:?}", err);
    } else {
      eprintln!("ERROR: {:?}", err);
    }
  }
}

fn try_main() -> AnyResult<()> {
  env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));
  // return image_decoding_speedrun::main();

  info!("{} v{} ({})", GAME_NAME, GAME_VERSION, GAME_ENGINE_NAME);
  info!("Initiating the boot sequence...");

  let game_fs = GameFs::init().context("Failed to initialize GameFs")?;

  let random =
    GlobalRandom::init().context("Failed to initialize a random number generator (RNG)")?;

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
  // TODO: A environment variable or a command-line flag for disabling the GL debug mode
  gl_attr.set_context_flags().debug().set();
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
    .allow_highdpi()
    .build()
    .context("Failed to create the game window")?;

  let sdl_gl_ctx = window
    .gl_create_context()
    .map_err(AnyError::msg)
    .context("Failed to create an OpenGL context for the game window")?;
  assert_eq!(
    (gl_attr.context_profile(), gl_attr.context_version()),
    (GL_CONTEXT_PROFILE, GL_CONTEXT_VERSION)
  );

  let gl = Rc::new(oogl::Context::load_with(|name| {
    video_subsystem.gl_get_proc_address(name) as *const c_void
  }));
  debug!("{:?}", gl.capabilities());

  gl.set_clear_color(BACKGROUND_COLOR);
  gl.clear(oogl::ClearFlags::COLOR);
  window.gl_swap_window();

  gl.set_blending_enabled(true);
  gl.set_blending_factors(oogl::BlendingFactor::SrcAlpha, oogl::BlendingFactor::OneMinusSrcAlpha);
  gl.set_blending_equation(oogl::BlendingEquation::Add);

  let globals = Rc::new({
    let window_size_i = Vec2::from(window.drawable_size());
    Globals {
      gl,
      game_fs,
      random,

      should_stop_game_loop: Cell::new(false),
      first_game_loop_tick: true,
      time: 0.0,
      delta_time: 0.0,
      fixed_time: 0.0,
      // TODO: reduce the timestep from 120 UPS to 60 UPS
      fixed_delta_time: 1.0 / 60.0 / 2.0,

      window_size_i,
      window_size: window_size_i.cast_into(),
      window_was_resized: true,
      window_is_focused: true,

      input_state: input::InputState::new(),
    }
  });

  let renderer = Renderer::init(globals.share()).context("Failed to initialize the renderer")?;
  let pong = Pong::init(globals.share()).context("Failed to initialize Pong")?;

  #[cfg(feature = "game_of_life")]
  let game_of_life =
    GameOfLife::init(globals.share()).context("Failed to initialize GameOfLife")?;
  #[cfg(feature = "mandelbrot")]
  let mandelbrot = Mandelbrot::init(globals.share()).context("Failed to initialize Mandelbrot")?;
  #[cfg(feature = "marching_squares")]
  let marching_squares =
    MarchingSquares::init(globals.share()).context("Failed to initialize MarchingSquares")?;

  globals.gl.release_shader_compiler();

  let mut game = Game {
    globals,

    sdl_context,
    video_subsystem,
    sdl_gl_ctx,
    window,
    event_pump,
    renderer,

    pong,
    #[cfg(feature = "game_of_life")]
    game_of_life,
    #[cfg(feature = "mandelbrot")]
    mandelbrot,
    #[cfg(feature = "marching_squares")]
    marching_squares,
  };

  info!("Core subsystems have been initialized, starting the game loop...");
  info!("Hi!");
  let result = game.start_loop().context("Critical error in the game loop!");
  info!("Bye!");
  result
}

struct Game {
  pub globals: SharedGlobals,

  pub sdl_context: sdl2::Sdl,
  pub video_subsystem: sdl2::VideoSubsystem,
  pub sdl_gl_ctx: sdl2::video::GLContext,
  pub window: Window,
  pub event_pump: EventPump,
  pub renderer: Renderer,

  pub pong: Pong,
  #[cfg(feature = "game_of_life")]
  pub game_of_life: GameOfLife,
  #[cfg(feature = "mandelbrot")]
  pub mandelbrot: Mandelbrot,
  #[cfg(feature = "marching_squares")]
  pub marching_squares: MarchingSquares,
}

impl Game {
  pub fn start_loop(&mut self) -> AnyResult<()> {
    let mut prev_time = Instant::now();
    let mut fixed_update_time_accumulator = 0.0;
    let mut idling = false;

    fn mut_globals(myself: &mut Game) -> &mut Globals {
      unsafe { Rc::get_mut_unchecked(&mut myself.globals) }
    }

    while !self.globals.should_stop_game_loop.get() {
      let current_time = Instant::now();
      let delta_time = (current_time - prev_time).as_secs_f64();
      {
        let globals = mut_globals(self);
        globals.delta_time = delta_time;
        globals.time += delta_time;
      }

      self.process_input();
      let window_is_focused = self.globals.window_is_focused;
      if window_is_focused {
        idling = false;
      }

      if !idling {
        #[cfg(feature = "gl_debug_all_commands")]
        println!("================ [OpenGL] ================");

        self.early_update().context("Error in early_update")?;

        fixed_update_time_accumulator += delta_time;
        let fixed_delta_time = self.globals.fixed_delta_time;
        // FIXME: What if a lot of time has passed between frames, e.g. due to the
        // game being suspended or paused with SIGSTOP (and resumed with SIGCONT)?
        while fixed_update_time_accumulator >= fixed_delta_time {
          mut_globals(self).fixed_time += fixed_delta_time;
          self.fixed_update().context("Error in fixed_update")?;
          fixed_update_time_accumulator -= fixed_delta_time;
        }

        self.update().context("Error in update")?;

        self.render().context("Error in render")?;
        self.window.gl_swap_window();
      } else {
        thread::sleep(Duration::from_secs_f64(GAME_LOOP_IDLING_WAIT_INTERVAL));
      }

      if !window_is_focused {
        idling = true;
      }

      mut_globals(self).first_game_loop_tick = false;
      prev_time = current_time;
    }

    Ok(())
  }

  pub fn process_input(&mut self) {
    let globals = unsafe { Rc::get_mut_unchecked(&mut self.globals) };
    let main_window = &mut self.window;
    let main_window_id = main_window.id();

    // This statement might seem weird at first, but it saves a us conditional
    // jump. It conveys the following logic:
    // - On the first game loop tick the window_was_resized flag is always set
    //   to true, so that everybody can configure up their internal state without
    //   having to do an extra "or first_game_loop_tick" check (which will be
    //   true literally once in the whole lifetime of the program).
    // - On all subsequent game loop ticks this flag has to be reset in order for
    //   its primary use to work. Here the fact that first_game_loop_tick will
    //   hold "false" is used to avoid branching.
    globals.window_was_resized = globals.first_game_loop_tick;

    globals.input_state.prev_mouse_pos = globals.input_state.mouse_pos;
    globals.input_state.prev_keyboard_state_table = globals.input_state.keyboard_state_table;

    for event in self.event_pump.poll_iter() {
      match event {
        Event::Quit { .. } | Event::AppTerminating { .. } | Event::AppLowMemory { .. } => {
          globals.should_stop_game_loop.set(true);
        }

        Event::Window { window_id, win_event: WindowEvent::Close, .. }
          if window_id == main_window_id =>
        {
          globals.should_stop_game_loop.set(true);
        }

        Event::Window { window_id, win_event: WindowEvent::FocusGained, .. }
          if window_id == main_window_id =>
        {
          globals.window_is_focused = true;
        }

        Event::Window { window_id, win_event: WindowEvent::FocusLost, .. }
          if window_id == main_window_id =>
        {
          globals.window_is_focused = false;
        }

        Event::Window { window_id, win_event: WindowEvent::Leave, .. }
          if window_id == main_window_id =>
        {
          globals.input_state.set_key_down(Key::MouseLeft, false);
          globals.input_state.set_key_down(Key::MouseMiddle, false);
          globals.input_state.set_key_down(Key::MouseRight, false);
          globals.input_state.set_key_down(Key::MouseX1, false);
          globals.input_state.set_key_down(Key::MouseX2, false);
        }

        Event::Window { window_id, win_event: WindowEvent::SizeChanged(..), .. }
          if window_id == main_window_id =>
        {
          let (w, h) = main_window.drawable_size();
          globals.window_size_i = vec2(w, h);
          globals.window_size = vec2(w as f32, h as f32);
          globals.window_was_resized = true;
        }

        Event::MouseMotion { window_id, x, y, .. } if window_id == main_window_id => {
          globals.input_state.mouse_pos =
            vec2(x as f32 - globals.window_size.x * 0.5, globals.window_size.y * 0.5 - y as f32);
        }

        Event::MouseButtonDown { window_id, mouse_btn, .. } if window_id == main_window_id => {
          if let Some(key) = Key::from_sdl2_mouse_button(mouse_btn) {
            globals.input_state.set_key_down(key, true);
          }
        }

        Event::MouseButtonUp { window_id, mouse_btn, .. } if window_id == main_window_id => {
          if let Some(key) = Key::from_sdl2_mouse_button(mouse_btn) {
            globals.input_state.set_key_down(key, false);
          }
        }

        Event::KeyDown { window_id, scancode: Some(scancode), .. }
          if window_id == main_window_id =>
        {
          if let Some(key) = Key::from_sdl2_scancode(scancode) {
            globals.input_state.set_key_down(key, true);
          }
        }

        Event::KeyUp { window_id, scancode: Some(scancode), .. }
          if window_id == main_window_id =>
        {
          if let Some(key) = Key::from_sdl2_scancode(scancode) {
            globals.input_state.set_key_down(key, false);
          }
        }

        _ => {}
      }
    }

    globals.input_state.delta_mouse_pos =
      globals.input_state.mouse_pos - globals.input_state.prev_mouse_pos;
  }

  pub fn early_update(&mut self) -> AnyResult<()> {
    if self.globals.input_state.is_key_pressed(Key::B) {
      breakpoint();
    }

    if self.globals.input_state.is_key_pressed(Key::Q) {
      self.globals.should_stop_game_loop.set(true);
    }

    #[cfg(not(feature = "disable_pong"))]
    self.pong.early_update();

    Ok(())
  }

  pub fn update(&mut self) -> AnyResult<()> {
    #[cfg(feature = "game_of_life")]
    self.game_of_life.update();

    #[cfg(feature = "mandelbrot")]
    self.mandelbrot.update();

    #[cfg(feature = "marching_squares")]
    self.marching_squares.update();

    Ok(())
  }

  pub fn fixed_update(&mut self) -> AnyResult<()> {
    #[cfg(not(feature = "disable_pong"))]
    self.pong.fixed_update();

    Ok(())
  }

  pub fn render(&mut self) -> AnyResult<()> {
    let gl = &self.globals.gl;
    if self.globals.window_was_resized {
      gl.set_viewport(vec2n(0), self.globals.window_size_i.cast_into());
    }
    gl.clear(oogl::ClearFlags::COLOR);

    #[cfg(feature = "game_of_life")]
    {
      self.game_of_life.render();
      self.renderer.prepare();
      self.game_of_life.render_debug_info(&mut self.renderer, &mut self.pong.font);
      self.renderer.finish();
    }

    #[cfg(feature = "mandelbrot")]
    {
      self.mandelbrot.render();
      self.renderer.prepare();
      self.mandelbrot.render_debug_info(&mut self.renderer, &mut self.pong.font);
      self.renderer.finish();
    }

    #[cfg(feature = "marching_squares")]
    {
      self.marching_squares.render();
      self.renderer.prepare();
      self.marching_squares.render_debug_info(&mut self.renderer, &mut self.pong.font);
      self.renderer.finish();
    }

    #[cfg(not(feature = "disable_pong"))]
    self.pong.render(&mut self.renderer);

    #[cfg(feature = "screenshot")]
    if self.globals.input_state.is_key_pressed(Key::F8) {
      self.screenshot().context("Failed to take a screenshot")?;
    }

    Ok(())
  }

  #[cfg(feature = "screenshot")]
  fn screenshot(&mut self) -> AnyResult<()> {
    let path = Path::new("screenshot.png");
    let size = self.globals.window_size_i;
    let mut pixels = vec![0; size.x as usize * size.y as usize * 4];
    info!(
      "Saving a screenshot to '{}', {}x{} RGBA, {} bytes for pixels",
      path.display(),
      size.x,
      size.y,
      pixels.len()
    );
    self.globals.gl.read_pixels_rgba(size, &mut pixels);
    flip_rgba_image_data_vertically(size, &mut pixels);

    let file =
      File::create(path).with_context(|| format!("Failed to create file '{}'", path.display()))?;
    let writer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(writer, size.x, size.y);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().context("Failed to write the PNG header")?;
    writer.write_image_data(&pixels).context("Failed to write the encoded PNG data")?;

    Ok(())
  }
}

#[cfg(feature = "screenshot")]
fn flip_rgba_image_data_vertically(size: Vec2u32, pixels: &mut [u8]) {
  assert!(pixels.len() == size.x as usize * size.y as usize * 4);

  let row_len = size.x as usize * 4;
  let (mut half1, mut half2) = pixels.split_at_mut(row_len * (size.y / 2) as usize);
  for _ in 0..size.y / 2 {
    // len needs to be saved because using the length in slicing expressions
    // makes the borrow checker unhappy
    let half2_len = half2.len();
    let row1 = &mut half1[..row_len];
    let row2 = &mut half2[half2_len - row_len..];
    row1.swap_with_slice(row2);
    half1 = &mut half1[row_len..];
    half2 = &mut half2[..half2_len - row_len];
  }
}
