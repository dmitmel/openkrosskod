use cardboard_math::*;
use cardboard_oogl as oogl;
use cardboard_oogl::traits::*;
use prelude_plus::*;

use crate::globals::SharedGlobals;
use crate::input::Key;
use crate::profiling::AverageTimeSampler;
use crate::renderer;

const CHUNK_SIZE: Vec2u8 = vec2n(128);
const CHUNKS_COUNT: Vec2u32 = vec2(8, 8);

const_assert!(CHUNK_SIZE.x as u32 * CHUNK_SIZE.y as u32 * 4 <= u16::MAX as u32 + 1);

const GRID_SIZE: Vec2u32 =
  vec2(CHUNK_SIZE.x as u32 * CHUNKS_COUNT.x, CHUNK_SIZE.y as u32 * CHUNKS_COUNT.y);

const SIMULATION_INTERVAL: f64 = 0.01;
const GLOBAL_COLOR: Colorf = colorn(1.0, 1.0);
const CELL_SIZE: Vec2f = vec2n(4.0);
const CELL_SPAWN_CHANCE: f32 = 0.25;
const CAMERA_ZOOM_SPEED: f32 = 0.04;
const CAMERA_MOVEMENT_SPEED_FROM_KEYBOARD: f32 = 1200.0;

fn simulation_rule(is_alive: bool, alive_neighbors: u8) -> bool {
  if is_alive {
    alive_neighbors == 2 || alive_neighbors == 3
  } else {
    alive_neighbors == 3
  }
}

#[repr(C, packed)]
#[derive(Copy, Debug, Clone, Default)]
struct Vertex {
  pos: Vec2u8,
}

#[derive(Debug)]
pub struct GameOfLife {
  globals: SharedGlobals,
  vertex_buf: oogl::VertexBuffer<Vertex>,
  chunk_textures: Vec<oogl::Texture2D<u8>>,
  program: oogl::Program,
  program_reflection: ProgramReflection,

  prev_simulation_time: f64,
  camera_pos: Vec2f,
  camera_zoom: f32,

  current_generation: Vec<u8>,
  next_generation: Vec<u8>,
  texture_data: Vec<u8>,

  simulation_times: AverageTimeSampler,
  texture_refill_times: AverageTimeSampler,
}

impl GameOfLife {
  pub fn init(globals: SharedGlobals) -> AnyResult<Self> {
    use oogl::ShaderType as ShTy;
    let vertex_shader =
      renderer::load_shader_asset(&globals, "shaders/game_of_life.vert.glsl", ShTy::Vertex)?;
    let fragment_shader =
      renderer::load_shader_asset(&globals, "shaders/game_of_life.frag.glsl", ShTy::Fragment)?;
    let mut program =
      renderer::load_program_asset(&globals, "GameOfLife", &[&vertex_shader, &fragment_shader])?;
    let program_reflection = ProgramReflection::new(&program);

    {
      let bound_program = program.bind();
      let reflection = &program_reflection;
      reflection.u_global_color.set(&bound_program, &GLOBAL_COLOR);
      reflection.u_cell_size.set(&bound_program, &CELL_SIZE);
      reflection.u_chunk_size.set(&bound_program, &Vec2f::cast_from(CHUNK_SIZE));
    }

    let mut vertex_buf = oogl::VertexBuffer::new(
      globals.gl.share(),
      oogl::BufferUsageHint::StaticDraw,
      vec![program_reflection.a_pos.to_pointer_simple_with_cast(oogl::AttribPtrTypeName::U8)],
    );
    vertex_buf.set_debug_label(b"GameOfLife.vertex_buf");
    vertex_buf.bind().alloc_and_set(&[
      Vertex { pos: vec2(0, 0) },
      Vertex { pos: vec2(1, 0) },
      Vertex { pos: vec2(1, 1) },
      Vertex { pos: vec2(0, 1) },
    ]);

    let texture_data = vec![0; CHUNK_SIZE.x as usize * CHUNK_SIZE.y as usize];
    let mut chunk_textures = Vec::with_capacity(CHUNKS_COUNT.x as usize * CHUNKS_COUNT.y as usize);
    for i in 0..chunk_textures.capacity() {
      let unit = None;
      let mut tex =
        oogl::Texture2D::new(globals.gl.share(), unit, oogl::TextureInputFormat::Luminance, None);
      tex.set_debug_label(format!("GameOfLife.chunk_textures[{}]", i).as_bytes());
      {
        let bound_tex = tex.bind(unit);
        bound_tex.set_size(Vec2u32::cast_from(CHUNK_SIZE));
        bound_tex.set_filters(oogl::TextureFilter::Nearest, None);
        bound_tex.alloc(0);
      }
      chunk_textures.push(tex);
    }

    let next_generation = vec![0; GRID_SIZE.x as usize * GRID_SIZE.y as usize];
    let current_generation = next_generation.clone();

    let mut myself = Self {
      globals,
      vertex_buf,
      chunk_textures,
      program,
      program_reflection,

      prev_simulation_time: 0.0,
      camera_pos: Vec2f::cast_from(GRID_SIZE) * CELL_SIZE * 0.5,
      camera_zoom: 1.0,

      current_generation,
      next_generation,
      texture_data,

      simulation_times: AverageTimeSampler::new(30),
      texture_refill_times: AverageTimeSampler::new(30),
    };
    myself.reset_simulation();
    myself.refill_textures();
    Ok(myself)
  }

  fn reset_simulation(&mut self) {
    for cell in &mut self.current_generation {
      *cell = (self.globals.random.next_f32() < CELL_SPAWN_CHANCE) as u8;
    }
  }

  fn current_generation_get(&self, pos: Vec2u32) -> u8 {
    self.current_generation[pos.x as usize + pos.y as usize * GRID_SIZE.x as usize]
  }

  fn next_generation_set(&mut self, pos: Vec2u32, value: u8) {
    self.next_generation[pos.x as usize + pos.y as usize * GRID_SIZE.x as usize] = value;
  }

  fn run_simulation(&mut self) {
    let start_time = Instant::now();

    for y in 0..GRID_SIZE.y {
      for x in 0..GRID_SIZE.x {
        let pos = vec2(x, y);
        let state: u8 = self.current_generation_get(pos);
        let is_alive = state > 0;

        let alive_neighbors: u8 = self.get_alive_neighbor_count(pos);
        let next_state: u8 =
          if simulation_rule(is_alive, alive_neighbors) { state.saturating_add(1) } else { 0 };

        self.next_generation_set(pos, next_state);
      }
    }

    mem::swap(&mut self.current_generation, &mut self.next_generation);

    self.simulation_times.push(start_time.elapsed());
  }

  fn get_alive_neighbor_count(&self, pos: Vec2u32) -> u8 {
    let mut alive_neighbors: u8 = 0;

    let mut visit_neighbor = |neighbor_pos: Vec2u32| {
      alive_neighbors += (self.current_generation_get(neighbor_pos) > 0) as u8;
    };

    let Vec2 { x, y } = pos;
    let can_inc_x = pos.x < GRID_SIZE.x - 1;
    let can_inc_y = pos.y < GRID_SIZE.y - 1;
    let can_dec_x = pos.x > 0;
    let can_dec_y = pos.y > 0;

    #[rustfmt::skip]
    {
      if can_inc_x              { visit_neighbor(vec2(x + 1, y    )) }
      if can_inc_x && can_inc_y { visit_neighbor(vec2(x + 1, y + 1)) }
      if              can_inc_y { visit_neighbor(vec2(x    , y + 1)) }
      if can_dec_x && can_inc_y { visit_neighbor(vec2(x - 1, y + 1)) }
      if can_dec_x              { visit_neighbor(vec2(x - 1, y    )) }
      if can_dec_x && can_dec_y { visit_neighbor(vec2(x - 1, y - 1)) }
      if              can_dec_y { visit_neighbor(vec2(x    , y - 1)) }
      if can_inc_x && can_dec_y { visit_neighbor(vec2(x + 1, y - 1)) }
    };

    alive_neighbors
  }

  fn refill_textures(&mut self) {
    let start_time = Instant::now();

    let mut texture_idx = 0;
    for chunk_y in 0..CHUNKS_COUNT.y {
      for chunk_x in 0..CHUNKS_COUNT.x {
        let chunk_pos = vec2(chunk_x, chunk_y);
        self.fill_chunk_texture(chunk_pos, texture_idx);
        texture_idx += 1;
      }
    }

    self.texture_refill_times.push(start_time.elapsed());
  }

  fn fill_chunk_texture(&mut self, chunk_pos: Vec2u32, texture: usize) {
    let chunk_contents_offset: Vec2u32 = chunk_pos * Vec2u32::cast_from(CHUNK_SIZE);

    let mut i = 0;
    for chunk_local_y in 0..CHUNK_SIZE.y {
      for chunk_local_x in 0..CHUNK_SIZE.x {
        let chunk_local_pos: Vec2u8 = vec2(chunk_local_x, chunk_local_y);
        let pos: Vec2u32 = Vec2u32::cast_from(chunk_local_pos) + chunk_contents_offset;
        let state = self.current_generation_get(pos);
        self.texture_data[i] = state;
        i += 1;
      }
    }

    let texture = &mut self.chunk_textures[texture];
    texture.bind(None).set(0, &self.texture_data);
  }

  pub fn update(&mut self) {
    if self.globals.input_state.is_key_pressed(Key::Space) {
      self.reset_simulation();
    }

    self.camera_zoom *=
      1.0 + self.globals.input_state.axis(Key::Minus, Key::Equals) as f32 * CAMERA_ZOOM_SPEED;

    let camera_movement = if self.globals.input_state.is_key_down(Key::MouseLeft) {
      -self.globals.input_state.delta_mouse_pos
    } else {
      let mut dir = Vec2f::ZERO;
      for &(key, movement) in &[
        (Key::W, Vec2f::UP),
        (Key::D, Vec2f::RIGHT),
        (Key::S, Vec2f::DOWN),
        (Key::A, Vec2f::LEFT),
      ] {
        if self.globals.input_state.is_key_down(key) {
          dir += movement;
        }
      }
      dir * CAMERA_MOVEMENT_SPEED_FROM_KEYBOARD * self.globals.delta_time as f32
    };
    self.camera_pos += camera_movement / self.camera_zoom;

    if self.globals.time >= self.prev_simulation_time + SIMULATION_INTERVAL {
      self.prev_simulation_time = self.globals.time;
      self.run_simulation();
    }
  }

  pub fn render(&mut self) {
    self.refill_textures();

    let bound_program = self.program.bind();
    let reflection = &self.program_reflection;

    reflection.u_camera_pos.set(&bound_program, &self.camera_pos);
    reflection.u_camera_zoom.set(&bound_program, &self.camera_zoom);
    if self.globals.window_was_resized {
      reflection.u_window_size.set(&bound_program, &self.globals.window_size);
    }

    let bound_vertex_buf = self.vertex_buf.bind();
    bound_vertex_buf.enable_attribs();
    bound_vertex_buf.configure_attribs();

    let mut texture_idx = 0;
    for chunk_y in 0..CHUNKS_COUNT.y {
      for chunk_x in 0..CHUNKS_COUNT.x {
        let chunk_contents_offset: Vec2u32 =
          vec2(chunk_x, chunk_y) * Vec2u32::cast_from(CHUNK_SIZE);
        reflection.u_chunk_offset.set(&bound_program, &Vec2f::cast_from(chunk_contents_offset));

        let bound_chunk_texture = self.chunk_textures[texture_idx].bind(None);
        reflection.u_chunk_texture.set(&bound_program, &bound_chunk_texture.unit());

        bound_vertex_buf.draw(&bound_program, oogl::DrawPrimitive::TriangleFan);

        texture_idx += 1;
      }
    }

    bound_vertex_buf.disable_attribs();
  }

  pub fn render_debug_info(
    &mut self,
    renderer: &mut renderer::Renderer,
    font: &mut renderer::Font,
  ) {
    let avg_texture_refill_time = self.texture_refill_times.average_micros() as f64 / 1000.0;
    let avg_simulation_time = self.simulation_times.average_micros() as f64 / 1000.0;

    let mut text_block_offset = Vec2f::ZERO;
    for &text in &[
      format!(" simulation time: {:.03?} ms", avg_simulation_time).as_str(),
      format!(" tex refill time: {:.03?} ms", avg_texture_refill_time).as_str(),
    ] {
      let text_block = &mut renderer::TextBlock {
        text,
        scale: vec2n(4.0),
        character_spacing: vec2n(0.4),
        horizontal_align: renderer::TextAlign::Start,
        vertical_align: renderer::TextAlign::Start,
      };
      let (text_block_size, char_size) = font.measure_size(text_block);
      let pos = (self.globals.window_size - char_size * 0.5) * vec2(-0.5, 0.5) - text_block_offset;
      renderer.draw_text(font, pos, text_block);
      text_block_offset.y += text_block_size.y;
    }
  }
}

oogl::program_reflection_block!({
  #[derive(Debug)]
  struct ProgramReflection {
    a_pos: oogl::Attrib<Vec2f>,

    u_global_color: oogl::Uniform<Colorf>,
    u_cell_size: oogl::Uniform<Vec2f>,
    u_chunk_size: oogl::Uniform<Vec2f>,

    u_window_size: oogl::Uniform<Vec2f>,
    u_camera_pos: oogl::Uniform<Vec2f>,
    u_camera_zoom: oogl::Uniform<f32>,

    u_chunk_offset: oogl::Uniform<Vec2f>,
    u_chunk_texture: oogl::Uniform<u16>,
  }
});
