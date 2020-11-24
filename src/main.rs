#![deny(missing_debug_implementations)]
#![allow(clippy::new_without_default, clippy::missing_safety_doc)]
#![feature(test, get_mut_unchecked)]

pub mod gen_idx; // TODO

pub mod game_fs;
pub mod globals;
pub mod image_decoding_speedrun;
pub mod input;
pub mod renderer;

use prelude_plus::*;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::video::{GLProfile, Window};
use sdl2::EventPump;

use cardboard_math::*;
use cardboard_oogl as oogl;

use crate::game_fs::*;
use crate::globals::*;
use crate::renderer::*;

const GAME_NAME: &str = "openKrossKod";
// const GAME_NAME: &str = env!("CARGO_PKG_NAME");
const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");
const GAME_ENGINE_NAME: &str = "Cardboard Engine, \"The Third Impact\" revision";

const GL_CONTEXT_PROFILE: GLProfile = GLProfile::GLES;
const GL_CONTEXT_VERSION: (u8, u8) = (2, 0);

const DEFAULT_WINDOW_SIZE: Vec2<u32> = vec2(568 * 2, 320 * 2);
const BACKGROUND_COLOR: Colorf = colorn(0.1, 1.0);

const GAME_LOOP_IDLING_WAIT_INTERVAL: f64 = 1.0 / 20.0;

const FONT_CHAR_GRID_SIZE: Vec2<u32> = vec2(4, 6);
const FONT_CHAR_SIZE: Vec2<u32> = vec2(3, 5);
const SCORE_LABEL_CHAR_SPACING: Vec2f = vec2n(1.0 / 3.0);
const SCORE_LABEL_TEXT_SCALE: Vec2f = vec2n(16.0);

const RACKET_SIZE: Vec2f = vec2(20.0, 200.0);
const RACKET_OFFSET: f32 = 2.0 * RACKET_SIZE.x + BALL_RADIUS;
const RACKET_COLOR: Colorf = colorn(0.9, 1.0);

const RACKET_MAX_SPEED: f32 = 1000.0;
const RACKET_ACCELERATION: f32 = 8.0;
const RACKET_SLOWDOWN: f32 = 12.0;
const RACKET_SPEED_EPSILON: f32 = 1.0;
const BOT_RACKET_VISION_DISTANCE: f32 = 1.0 / 2.0;

const BALL_RADIUS: f32 = 40.0;
const BALL_ROTATION_SPEED: f32 = 1.0;
const BALL_MAX_SPEED: f32 = 1400.0;
const BALL_MAX_VEL_DEVIATION_ANGLE: f32 = (/* 90 deg */f32::consts::FRAC_PI_2) * (2.0 / 3.0);
const BALL_THROW_DISTANCE_FROM_RACKET: f32 = RACKET_SIZE.y;

fn breakpoint() {
  use nix::sys::signal;
  signal::raise(signal::SIGINT).unwrap();
}

fn main() {
  if let Err(err) = try_main() {
    if log_enabled!(LogLevel::Error) {
      error!("{:?}", err);
    } else {
      eprintln!("ERROR: ${:?}", err);
    }
  }
}

fn try_main() -> AnyResult<()> {
  env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));
  // return image_decoding_speedrun::main();

  info!("{} v{} ({})", GAME_NAME, GAME_VERSION, GAME_ENGINE_NAME);

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
  gl.set_clear_color(BACKGROUND_COLOR);
  gl.clear(oogl::ClearFlags::COLOR);

  gl.set_blending_enabled(true);
  gl.set_blending_factors(oogl::BlendingFactor::SrcAlpha, oogl::BlendingFactor::OneMinusSrcAlpha);
  gl.set_blending_equation(oogl::BlendingEquation::Add);

  let globals = Rc::new({
    let window_size_i = Vec2::from(window.size());
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

  let (ball_texture, _ball_texture_size) =
    load_texture_asset(&globals, "ball.png", oogl::TextureFilter::Linear)?;
  let (font_texture, font_texture_size) =
    load_texture_asset(&globals, "font.png", oogl::TextureFilter::Nearest)?;

  let renderer =
    Renderer::init(Rc::clone(&globals)).context("Failed to initialize the renderer")?;

  let state = {
    let (mut left_racket, mut right_racket) = (Racket::new(1, -1.0), Racket::new(2, 1.0));
    left_racket.update_pos(&globals);
    right_racket.update_pos(&globals);

    let (left_racket_controller, right_racket_controller) = (
      // RacketController::Player { key_up: Scancode::W, key_down: Scancode::S },
      RacketController::Bot,
      RacketController::Player { key_up: Scancode::Up, key_down: Scancode::Down },
    );

    let mut ball = Ball::new(3);
    ball.throw_at(&globals, if globals.random.next_bool() { &left_racket } else { &right_racket });

    GameState { left_racket, left_racket_controller, right_racket, right_racket_controller, ball }
  };

  let mut game = Game {
    globals,
    state,

    sdl_context,
    video_subsystem,
    window,
    event_pump,
    renderer,

    ball_texture,
    font: Font {
      texture: font_texture,
      texture_size: font_texture_size,
      grid_size: vec2(16, 8),
      grid_cell_size: FONT_CHAR_GRID_SIZE,
      character_size: FONT_CHAR_SIZE,
    },

    debug_vectors: Vec::new(),
  };

  let result = game.start_loop();
  info!("Bye!");
  result
}

#[derive(Debug)]
struct GameState {
  pub left_racket: Racket,
  pub left_racket_controller: RacketController,
  pub right_racket: Racket,
  pub right_racket_controller: RacketController,
  pub ball: Ball,
}

type EntityId = u32;

#[derive(Debug, Default)]
struct CollEntry {
  pub id: EntityId,
  pub size: Vec2f,
  pub pos: Vec2f,
  pub vel: Vec2f,
  pub accel: Vec2f,
  pub slowdown: f32,
  pub max_speed: f32,
}

impl CollEntry {
  fn extents(&self) -> Vec2f { self.size / 2.0 }
}

#[derive(Debug)]
struct Racket {
  pub coll: CollEntry,
  pub side: f32,
  pub score: u32,
}

impl Racket {
  fn new(id: EntityId, side: f32) -> Self {
    Self {
      coll: CollEntry {
        id,
        size: RACKET_SIZE,
        max_speed: RACKET_MAX_SPEED,
        slowdown: RACKET_SLOWDOWN,
        ..Default::default()
      },
      side,
      score: 0,
    }
  }

  fn update_pos(&mut self, globals: &Globals) {
    self.coll.pos.x =
      self.side * (globals.window_size.x / 2.0 - self.coll.size.x / 2.0 - RACKET_OFFSET);
  }
}

#[derive(Debug)]
struct Ball {
  pub coll: CollEntry,
  pub rotation: f32,
  pub rotation_speed: f32,
  pub currently_colliding_with: Option<EntityId>,
}

impl Ball {
  fn new(id: EntityId) -> Self {
    Ball {
      coll: CollEntry {
        id,
        size: vec2n(BALL_RADIUS * 2.0),
        vel: vec2n(0.0),
        max_speed: BALL_MAX_SPEED,
        ..Default::default()
      },
      rotation: 0.0,
      rotation_speed: BALL_ROTATION_SPEED,
      currently_colliding_with: None,
    }
  }

  fn throw_at(&mut self, globals: &Globals, racket: &Racket) {
    let dist = BALL_THROW_DISTANCE_FROM_RACKET;
    self.coll.pos = racket.coll.pos - vec2(racket.side * (dist + racket.coll.size.x / 2.0), 0.0);

    let max_angle = ((racket.coll.size.y / 2.0) / dist).atan();
    let angle = (globals.random.next_f32() * 2.0 - 1.0) * max_angle;
    self.coll.vel = vec2(racket.side, 0.0).rotated(angle) * BALL_MAX_SPEED;
  }
}

#[derive(Debug)]
enum RacketController {
  Player { key_up: Scancode, key_down: Scancode },
  Bot,
}

impl RacketController {
  fn get_movement_direction(&mut self, globals: &Globals, racket: &Racket, ball: &Ball) -> f32 {
    let mut dir = 0.0;

    match self {
      Self::Player { key_up, key_down } => {
        if globals.input_state.is_key_down(*key_up) {
          dir += 1.0;
        }
        if globals.input_state.is_key_down(*key_down) {
          dir -= 1.0;
        }
      }

      Self::Bot => {
        let vision_dist = BOT_RACKET_VISION_DISTANCE * globals.window_size.x;
        if ball.coll.vel.x * racket.side > 0.0
          && (racket.coll.pos.x - ball.coll.pos.x).abs() <= vision_dist
        {
          dir = (ball.coll.pos.y - racket.coll.pos.y).signum();
        }
      }
    }

    dir
  }
}

struct Game {
  pub globals: SharedGlobals,
  pub state: GameState,

  pub sdl_context: sdl2::Sdl,
  pub video_subsystem: sdl2::VideoSubsystem,
  pub window: Window,
  pub event_pump: EventPump,
  pub renderer: Renderer,

  pub ball_texture: oogl::Texture2D,
  pub font: Font,

  pub debug_vectors: Vec<(Vec2f, Vec2f, Colorf)>,
}

impl Game {
  pub fn start_loop(&mut self) -> AnyResult<()> {
    let mut prev_time = Instant::now();
    let mut fixed_update_time_accumulator = 0.0;
    let mut idling = false;

    fn mut_globals(this: &mut Game) -> &mut Globals {
      unsafe { Rc::get_mut_unchecked(&mut this.globals) }
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
        self.early_update();

        fixed_update_time_accumulator += delta_time;
        let fixed_delta_time = self.globals.fixed_delta_time;
        // FIXME: What if a lot of time has passed between frames, e.g. due to the
        // game being suspended or paused with SIGSTOP (and resumed with SIGCONT)?
        while fixed_update_time_accumulator >= fixed_delta_time {
          mut_globals(self).fixed_time += fixed_delta_time;
          self.fixed_update();
          fixed_update_time_accumulator -= fixed_delta_time;
        }

        self.update();

        if cfg!(feature = "gl_debug_all_commands") {
          println!("================ [OpenGL] ================");
        }

        self.render();
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
    let main_window_id = self.window.id();

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

        Event::KeyDown { window_id, scancode: Some(scancode), .. }
          if window_id == main_window_id =>
        {
          globals.input_state.keyboard.set(scancode, true);
        }

        Event::KeyUp { window_id, scancode: Some(scancode), .. }
          if window_id == main_window_id =>
        {
          globals.input_state.keyboard.set(scancode, false);
        }

        _ => {}
      }
    }
  }

  pub fn early_update(&mut self) {
    if self.globals.input_state.is_key_down(Scancode::B) {
      breakpoint();
    }

    if self.globals.input_state.is_key_down(Scancode::Q) {
      self.globals.should_stop_game_loop.set(true);
    }

    if self.globals.window_was_resized {
      for racket in &mut [&mut self.state.left_racket, &mut self.state.right_racket] {
        racket.update_pos(&self.globals);
      }
    }
  }

  pub fn update(&mut self) {}

  pub fn fixed_update(&mut self) {
    self.debug_vectors.clear();

    let fixed_delta_time = self.globals.fixed_delta_time as f32;
    let window_size = self.globals.window_size;

    let GameState {
      ball,
      left_racket,
      left_racket_controller,
      right_racket,
      right_racket_controller,
      ..
    } = &mut self.state;

    for (racket, controller) in &mut [
      (&mut *left_racket, &mut *left_racket_controller),
      (&mut *right_racket, &mut *right_racket_controller),
    ] {
      let dir = controller.get_movement_direction(&self.globals, &racket, &ball);
      racket.coll.accel.y = dir * RACKET_MAX_SPEED * RACKET_ACCELERATION;
    }

    {
      let window_bouncing_bounds: Vec2f = window_size / 2.0 - ball.coll.size / 2.0;

      if ball.coll.pos.x.abs() >= window_bouncing_bounds.x + ball.coll.size.x * 2.0 {
        let winner_racket =
          if ball.coll.pos.x >= 0.0 { &mut *left_racket } else { &mut *right_racket };
        winner_racket.score += 1;
        ball.throw_at(&self.globals, winner_racket);
      }

      if ball.coll.pos.y.abs() >= window_bouncing_bounds.y {
        ball.coll.vel.y = -ball.coll.vel.y;
      }

      ball.rotation += ball.rotation_speed * f32::consts::TAU * fixed_delta_time;

      let vel = &mut ball.coll.vel;
      let vel_magnitude = vel.magnitude();
      if vel_magnitude != 0.0 {
        let vel_guide = vec2(vel.x.signum(), 0.0);
        let vel_angle = vel_guide.angle_normalized(*vel / vel_magnitude);
        if vel_angle >= BALL_MAX_VEL_DEVIATION_ANGLE {
          let sign = vel_guide.angle_sign(*vel);
          *vel = vel_guide.rotated(BALL_MAX_VEL_DEVIATION_ANGLE * sign) * vel_magnitude;
        }
      }
    }

    for coll in &mut [&mut left_racket.coll, &mut right_racket.coll, &mut ball.coll] {
      coll.vel += if !coll.accel.is_zero() { coll.accel } else { -coll.vel * coll.slowdown }
        * fixed_delta_time;

      coll.vel = coll.vel.clamp_magnitude(coll.max_speed);
      if coll.vel.sqr_magnitude() < RACKET_SPEED_EPSILON * RACKET_SPEED_EPSILON {
        coll.vel = vec2n(0.0);
      }

      coll.pos += coll.vel * fixed_delta_time;
      coll.pos.y = coll.pos.y.clamp2_abs((window_size.y / 2.0 - coll.size.y / 2.0).abs())
    }

    // dbg!(left_racket.coll.vel, right_racket.coll.vel);

    // TODO: Rewrite the collision handling system. The current one leaves a lot
    // to be desired.
    for racket in &mut [&mut *left_racket, &mut *right_racket] {
      // <https://gamedev.stackexchange.com/q/136073/145058>
      let racket_extents = racket.coll.extents();

      let mut coll_point = (ball.coll.pos - racket.coll.pos).clamp2_abs(racket_extents);
      // self.debug_vectors.push((racket.coll.pos, coll_point, color(0.0, 1.0, 0.0, 0.5)));

      // <https://stackoverflow.com/a/10657968/12005228>
      if (coll_point.x / racket_extents.x).abs() >= (coll_point.y / racket_extents.y).abs() {
        coll_point.x = coll_point.x.signum() * racket_extents.x; // vertical edge
      } else {
        coll_point.y = coll_point.y.signum() * racket_extents.y; // horizontal edge
      }
      // self.debug_vectors.push((racket.coll.pos, coll_point, color(1.0, 0.0, 0.0, 0.5)));

      coll_point += racket.coll.pos;

      let coll_dir = ball.coll.pos - coll_point;
      // self.debug_vectors.push((coll_point, coll_dir, color(0.0, 0.0, 1.0, 0.5)));

      let coll_dir_magnitude = coll_dir.magnitude();

      let current_collider = &mut ball.currently_colliding_with;
      if current_collider.map_or(true, |id| id == racket.coll.id) {
        // TODO: handle situations when the ball clips inside of the racket
        if coll_dir_magnitude <= BALL_RADIUS {
          if current_collider.is_none() {
            *current_collider = Some(racket.coll.id);

            if coll_dir_magnitude > 0.0 {
              ball.coll.vel = ball.coll.vel.reflected_normal(coll_dir / coll_dir_magnitude);
            } else {
              todo!();
            }

            ball.coll.vel =
              (ball.coll.vel + racket.coll.vel).with_magnitude(ball.coll.vel.magnitude());
          }
        } else {
          *current_collider = None;
        }
      }
    }
  }

  pub fn render(&mut self) {
    let gl = &self.globals.gl;
    if self.globals.window_was_resized {
      gl.set_viewport(vec2n(0), self.globals.window_size_i.cast_into());
    }
    gl.clear(oogl::ClearFlags::COLOR);

    self.renderer.prepare();
    let window_size = self.globals.window_size;

    let GameState { ball, left_racket, right_racket, .. } = &self.state;

    for (text, side, align) in &[
      (format!("{}", left_racket.score).as_str(), -1.0, TextAlign::End),
      (":", 0.0, TextAlign::Center),
      (format!("{}", right_racket.score).as_str(), 1.0, TextAlign::Start),
    ] {
      let text_block = &mut TextBlock {
        text,
        scale: SCORE_LABEL_TEXT_SCALE,
        character_spacing: SCORE_LABEL_CHAR_SPACING,
        horizontal_align: *align,
        vertical_align: TextAlign::Start,
      };
      let (_text_block_size, char_spacing) = self.font.measure_size(text_block);
      let pos = vec2(side * char_spacing.x / 2.0, window_size.y / 2.0);
      self.renderer.draw_text(&mut self.font, pos, text_block);
    }

    for racket in &[left_racket, right_racket] {
      self.renderer.draw_shape(&mut Shape {
        type_: ShapeType::Rectangle,
        pos: racket.coll.pos,
        size: racket.coll.size,
        rotation: 0.0,
        fill: ShapeFill::Color(RACKET_COLOR),
        fill_clipping: None,
      });
    }

    self.renderer.draw_shape(&mut Shape {
      type_: ShapeType::Ellipse,
      pos: ball.coll.pos,
      size: ball.coll.size,
      rotation: ball.rotation,
      fill: ShapeFill::Texture(&mut self.ball_texture),
      fill_clipping: None,
    });

    // TODO: remove
    for &(start_point, vector, color) in &self.debug_vectors {
      let angle = vector.angle_from_x_axis();

      self.renderer.draw_shape(&mut Shape {
        type_: ShapeType::Rectangle,
        pos: start_point + vector / 2.0,
        size: vec2(vector.magnitude(), 5.0),
        rotation: angle,
        fill: ShapeFill::Color(color),
        fill_clipping: None,
      });

      self.renderer.draw_shape(&mut Shape {
        type_: ShapeType::Rectangle,
        pos: start_point + vector,
        size: vec2n(32.0),
        rotation: angle,
        fill: ShapeFill::Color(color),
        fill_clipping: None,
      });
    }

    if !self.globals.window_is_focused {
      self.renderer.draw_shape(&mut Shape {
        type_: ShapeType::Rectangle,
        pos: vec2n(0.0),
        size: window_size,
        rotation: 0.0,
        fill: ShapeFill::Color(color(0.0, 0.0, 0.0, 0.6)),
        fill_clipping: None,
      });
    }
  }
}
