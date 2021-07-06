use cardboard_math::*;
use cardboard_oogl as oogl;
use prelude_plus::*;

use crate::globals::{Globals, SharedGlobals};
use crate::input::Key;
use crate::renderer;

const FONT_CHAR_GRID_SIZE: Vec2u32 = vec2(4, 6);
const FONT_CHAR_SIZE: Vec2u32 = vec2(3, 5);
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

#[derive(Debug)]
struct GameState {
  left_racket: Racket,
  left_racket_controller: RacketController,
  right_racket: Racket,
  right_racket_controller: RacketController,
  ball: Ball,
}

type EntityId = u32;

#[derive(Debug, Default)]
struct CollEntry {
  id: EntityId,
  size: Vec2f,
  pos: Vec2f,
  vel: Vec2f,
  accel: Vec2f,
  slowdown: f32,
  max_speed: f32,
}

impl CollEntry {
  fn extents(&self) -> Vec2f { self.size / 2.0 }
}

#[derive(Debug)]
struct Racket {
  coll: CollEntry,
  side: f32,
  score: u32,
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
  coll: CollEntry,
  rotation: f32,
  rotation_speed: f32,
  currently_colliding_with: Option<EntityId>,
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
  Player { key_up: Key, key_down: Key },
  Bot,
}

impl RacketController {
  fn get_movement_direction(&mut self, globals: &Globals, racket: &Racket, ball: &Ball) -> f32 {
    match self {
      Self::Player { key_up, key_down } => globals.input_state.axis(*key_down, *key_up) as f32,

      Self::Bot => {
        let vision_dist = BOT_RACKET_VISION_DISTANCE * globals.window_size.x;
        if ball.coll.vel.x * racket.side > 0.0
          && (racket.coll.pos.x - ball.coll.pos.x).abs() <= vision_dist
        {
          (ball.coll.pos.y - racket.coll.pos.y).signum()
        } else {
          0.0
        }
      }
    }
  }
}

#[derive(Debug)]
pub struct Pong {
  globals: SharedGlobals,
  state: GameState,
  debug_vectors: Vec<(Vec2f, Vec2f, Colorf)>,
  pub font: renderer::Font,
  ball_texture: oogl::Texture2D,
}

impl Pong {
  pub fn init(globals: SharedGlobals) -> AnyResult<Self> {
    let state = {
      let (mut left_racket, mut right_racket) = (Racket::new(1, -1.0), Racket::new(2, 1.0));
      left_racket.update_pos(&globals);
      right_racket.update_pos(&globals);

      let (left_racket_controller, right_racket_controller) = (
        // RacketController::Player { key_up: Key::W, key_down: Key::S },
        RacketController::Bot,
        RacketController::Player { key_up: Key::Up, key_down: Key::Down },
      );

      let mut ball = Ball::new(3);
      ball
        .throw_at(&globals, if globals.random.next_bool() { &left_racket } else { &right_racket });

      GameState {
        left_racket,
        left_racket_controller,
        right_racket,
        right_racket_controller,
        ball,
      }
    };

    let font_texture =
      renderer::load_texture_asset(&globals, "font.png", oogl::TextureFilter::Nearest)?;
    let ball_texture =
      renderer::load_texture_asset(&globals, "ball.png", oogl::TextureFilter::Linear)?;

    Ok(Self {
      globals,
      state,
      debug_vectors: Vec::new(),
      font: renderer::Font {
        texture: font_texture,
        grid_size: vec2(16, 8),
        grid_cell_size: FONT_CHAR_GRID_SIZE,
        character_size: FONT_CHAR_SIZE,
      },
      ball_texture,
    })
  }

  pub fn early_update(&mut self) {
    if self.globals.window_was_resized {
      for racket in &mut [&mut self.state.left_racket, &mut self.state.right_racket] {
        racket.update_pos(&self.globals);
      }
    }
  }

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
      let dir = controller.get_movement_direction(&self.globals, racket, ball);
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

      ball.rotation += ball.rotation_speed * fixed_delta_time * f32::consts::TAU;
      ball.rotation %= f32::consts::TAU;

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

  pub fn render(&mut self, renderer: &mut renderer::Renderer) {
    use renderer::{Shape, ShapeFill, ShapeType, TextAlign, TextBlock};
    renderer.prepare();
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
      renderer.draw_text(&mut self.font, pos, text_block);
    }

    for racket in &[left_racket, right_racket] {
      renderer.draw_shape(&mut Shape {
        type_: ShapeType::Rectangle,
        pos: racket.coll.pos,
        size: racket.coll.size,
        rotation: 0.0,
        fill: ShapeFill::Color(RACKET_COLOR),
        fill_clipping: None,
      });
    }

    renderer.draw_shape(&mut Shape {
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

      renderer.draw_shape(&mut Shape {
        type_: ShapeType::Rectangle,
        pos: start_point + vector / 2.0,
        size: vec2(vector.magnitude(), 5.0),
        rotation: angle,
        fill: ShapeFill::Color(color),
        fill_clipping: None,
      });

      renderer.draw_shape(&mut Shape {
        type_: ShapeType::Rectangle,
        pos: start_point + vector,
        size: vec2n(32.0),
        rotation: angle,
        fill: ShapeFill::Color(color),
        fill_clipping: None,
      });
    }

    if !self.globals.window_is_focused {
      renderer.draw_shape(&mut Shape {
        type_: ShapeType::Rectangle,
        pos: vec2n(0.0),
        size: window_size,
        rotation: 0.0,
        fill: ShapeFill::Color(color(0.0, 0.0, 0.0, 0.6)),
        fill_clipping: None,
      });
    }

    renderer.finish();
  }
}
