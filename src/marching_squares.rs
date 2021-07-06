// <https://en.wikipedia.org/wiki/Marching_squares>

use cardboard_math::*;
use cardboard_oogl as oogl;
use cardboard_oogl::traits::*;
use prelude_plus::*;

use crate::globals::SharedGlobals;
use crate::input::Key;
use crate::profiling::AverageTimeSampler;
use crate::renderer::*;

const GRID_SIZE: Vec2u32 = vec2(128, 64);
// const GRID_SIZE: Vec2u32 = vec2(64, 32);
// const GRID_SIZE: Vec2u32 = vec2(32, 16);
const OPACITY: f32 = 1.0;

#[repr(C, packed)]
#[derive(Copy, Debug, Clone, Default)]
struct Vertex {
  pos: Vec2f,
  pos_inside_square: Vec2f,
  corner_values: Vec4f,
}

#[derive(Debug)]
pub struct MarchingSquares {
  globals: SharedGlobals,
  vbo: oogl::VertexBuffer<Vertex>,
  ebo: oogl::ElementBuffer<u16>,
  program: oogl::Program,
  program_reflection: ProgramReflection,

  prev_grid_size: Vec2u32,
  grid_size: Vec2u32,
  current_row_values: Vec<f32>,
  next_row_values: Vec<f32>,
  mesh_vertices: Vec<Vertex>,
  mesh_indices: Vec<u16>,

  mesh_rebuild_times: AverageTimeSampler,

  isovalue: f32,
  field_function: Box<dyn FieldFunction>,
}

impl MarchingSquares {
  pub fn init(globals: SharedGlobals) -> AnyResult<Self> {
    use oogl::ShaderType;
    let vertex_shader =
      load_shader_asset(&globals, "shaders/marching_squares.vert.glsl", ShaderType::Vertex)?;
    let fragment_shader =
      load_shader_asset(&globals, "shaders/marching_squares.frag.glsl", ShaderType::Fragment)?;
    let program =
      load_program_asset(&globals, "MarchingSquares", &[&vertex_shader, &fragment_shader])?;
    let program_reflection = ProgramReflection::new(&program);

    let vbo_attribs = vec![
      program_reflection.a_pos.to_pointer_simple(),
      program_reflection.a_pos_inside_square.to_pointer_simple(),
      program_reflection.a_corner_values.to_pointer_simple(),
    ];

    let buf_usage_hint = oogl::BufferUsageHint::StreamDraw;

    let vbo = oogl::VertexBuffer::new(globals.gl.share(), buf_usage_hint, vbo_attribs);
    vbo.set_debug_label(b"MarchingSquares.vbo");

    let ebo = oogl::ElementBuffer::new(globals.gl.share(), buf_usage_hint);
    ebo.set_debug_label(b"MarchingSquares.ebo");

    let grid_size = GRID_SIZE;

    let field_function = Box::new(TestFieldFunction::new(globals.share()));

    Ok(Self {
      globals,
      vbo,
      ebo,
      program,
      program_reflection,

      prev_grid_size: vec2n(0),
      grid_size,
      current_row_values: Vec::new(),
      next_row_values: Vec::new(),
      mesh_vertices: Vec::new(),
      mesh_indices: Vec::new(),

      mesh_rebuild_times: AverageTimeSampler::new(30),

      field_function,
      isovalue: 0.5,
    })
  }

  fn rebuild_mesh(&mut self) {
    let start_time = Instant::now();
    let grid_size = self.grid_size;

    if self.prev_grid_size != grid_size {
      self.prev_grid_size = grid_size;

      self.current_row_values = vec![0.0; grid_size.x as usize + 1];
      self.next_row_values = self.current_row_values.clone();
    }

    self.mesh_vertices.clear();
    self.mesh_indices.clear();

    self.field_function.prepare(grid_size);

    self.calculate_next_row(0);
    for y in 0..grid_size.y {
      mem::swap(&mut self.current_row_values, &mut self.next_row_values);
      self.calculate_next_row(y + 1);
      for x in 0..grid_size.x {
        self.calculate_square(vec2(x, y));
      }
    }

    self.mesh_rebuild_times.push(start_time.elapsed());
  }

  fn calculate_next_row(&mut self, y: u32) {
    let mut first_point_offset = Vec2f::cast_from(self.grid_size) * -0.5;
    first_point_offset.y += y as f32;
    let increment_per_point = Vec2f::RIGHT;
    self.field_function.get_row(
      first_point_offset,
      increment_per_point,
      &mut self.next_row_values,
    );
  }

  pub fn calculate_square(&mut self, square_pos: Vec2u32) {
    let isovalue = self.isovalue;

    #[derive(Debug, Default, Clone, Copy)]
    struct Corner {
      state: bool,
      value: f32,
      pos_f: Vec2f,
      offset_f: Vec2f,
    }

    let mut corners = [Corner::default(); 4];
    let mut corner_values = vec4n(0.0);
    let mut square_config = 0u8;

    const CORNER_OFFSETS: [Vec2u32; 4] = [vec2(0, 0), vec2(1, 0), vec2(1, 1), vec2(0, 1)];

    for (i, &corner_offset) in CORNER_OFFSETS.iter().enumerate() {
      let corner_pos = square_pos + corner_offset;

      let row_values: &[f32] =
        [&self.current_row_values, &self.next_row_values][corner_offset.y as usize];
      let value: f32 = row_values[corner_pos.x as usize];
      corner_values[i] = value;

      let state = value >= isovalue;

      square_config = (square_config << 1) | (state as u8);
      corners[i] = Corner {
        state,
        value,
        pos_f: Vec2f::cast_from(corner_pos),
        offset_f: Vec2f::cast_from(corner_offset),
      }
    }

    const STARTING_CORNER_PER_CONFIG: [u8; 1 << 4] =
      [0, 3, 2, 2, 1, 0, 2, 1, 0, 0, 1, 2, 0, 3, 0, 0];
    let mut starting_corner = STARTING_CORNER_PER_CONFIG[square_config as usize] as usize;

    let mut vertex_index = self.mesh_vertices.len() as u16;
    let first_vertex_index = vertex_index;
    let mut pushed_vertices_count = 0u16;
    let mut triangle_strip_mode = true;

    if square_config == 0b0101 || square_config == 0b1010 {
      // TODO: sample the function
      let average_corner_value =
        corners.iter().map(|c| c.value).sum::<f32>() / corners.len() as f32;
      triangle_strip_mode = average_corner_value >= isovalue;
      if triangle_strip_mode {
        starting_corner = starting_corner.wrapping_add(1);
      }
    }

    let mut push_vertex = |pos: Vec2f, pos_inside_square: Vec2f| {
      if triangle_strip_mode && pushed_vertices_count > 2 {
        self.mesh_indices.push(first_vertex_index);
        self.mesh_indices.push(vertex_index - 1);
      }

      self.mesh_vertices.push(Vertex { pos, pos_inside_square, corner_values });
      self.mesh_indices.push(vertex_index);
      vertex_index += 1;
      pushed_vertices_count += 1;
    };

    for mut corner_i in 0..corners.len() {
      corner_i = corner_i.wrapping_add(starting_corner);
      let corner = corners[corner_i % corners.len()];
      let next_corner = corners[corner_i.wrapping_add(1) % corners.len()];

      if corner.state {
        push_vertex(corner.pos_f, corner.offset_f);
      }

      if corner.state != next_corner.state {
        let lerp_factor = ((isovalue - corner.value) / (corner.value - next_corner.value)).abs();
        push_vertex(
          corner.pos_f.lerp(next_corner.pos_f, lerp_factor),
          corner.offset_f.lerp(next_corner.offset_f, lerp_factor),
        );
      }
    }
  }

  pub fn update(&mut self) {
    self.isovalue += 0.25
      * self.globals.input_state.axis(Key::Minus, Key::Equals) as f32
      * self.globals.delta_time as f32;
  }

  pub fn render(&mut self) {
    self.rebuild_mesh();

    let bound_program = self.program.bind();
    let reflection = &self.program_reflection;
    reflection.u_grid_size.set(&bound_program, &Vec2f::cast_from(self.grid_size));
    reflection.u_global_color.set(&bound_program, &colorn(1.0, OPACITY));

    {
      let bound_vbo = self.vbo.bind();
      bound_vbo.enable_attribs();
      bound_vbo.configure_attribs();
      let bound_ebo = self.ebo.bind();

      copy_data_from_vec_into_buffer(&bound_vbo, &self.mesh_vertices);
      copy_data_from_vec_into_buffer(&bound_ebo, &self.mesh_indices);

      let indices_count = self.mesh_indices.len();
      bound_ebo.draw_slice(&bound_program, oogl::DrawPrimitive::Points, ..indices_count);
      bound_ebo.draw_slice(&bound_program, oogl::DrawPrimitive::Triangles, ..indices_count);

      bound_vbo.disable_attribs();
    }
  }

  pub fn render_debug_info(&mut self, renderer: &mut Renderer, font: &mut Font) {
    let vertices = &self.mesh_vertices;
    let indices = &self.mesh_indices;

    let avg_mesh_rebuild_time = self.mesh_rebuild_times.average_micros() as f64 / 1000.0;

    let mut text_block_offset = Vec2f::ZERO;
    for &text in &[
      format!(" mesh rebuild time: {:.03?} ms", avg_mesh_rebuild_time).as_str(),
      format!("  vertices: {:>5}/{:<5}", vertices.len(), vertices.capacity()).as_str(),
      format!("   indices: {:>5}/{:<5}", indices.len(), indices.capacity()).as_str(),
      format!(" triangles: {:>5}/{:<5}", indices.len() / 3, indices.capacity() / 3).as_str(),
    ] {
      let text_block = &mut TextBlock {
        text,
        scale: vec2n(4.0),
        character_spacing: vec2n(0.4),
        horizontal_align: TextAlign::Start,
        vertical_align: TextAlign::Start,
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
    a_pos_inside_square: oogl::Attrib<Vec2f>,
    a_corner_values: oogl::Attrib<Vec4f>,
    u_grid_size: oogl::Uniform<Vec2f>,
    u_global_color: oogl::Uniform<Colorf>,
  }
});

pub trait FieldFunction: fmt::Debug {
  fn prepare(&mut self, grid_size: Vec2u32);
  fn get(&mut self, point: Vec2f) -> f32;

  fn get_row(&mut self, first_point_offset: Vec2f, increment_per_point: Vec2f, out: &mut [f32]) {
    for (x, out_value) in out.iter_mut().enumerate() {
      let point = vec2n(x as f32) * increment_per_point + first_point_offset;
      *out_value = self.get(point);
    }
  }
}

#[derive(Debug)]
struct TestFieldFunction {
  globals: SharedGlobals,
  grid_size_f: Vec2f,
  metaball_pos: Vec2f,
}

impl TestFieldFunction {
  fn new(globals: SharedGlobals) -> Self {
    Self { globals, grid_size_f: Vec2f::ZERO, metaball_pos: Vec2f::ZERO }
  }
}

impl FieldFunction for TestFieldFunction {
  fn prepare(&mut self, grid_size: Vec2u32) {
    self.grid_size_f = Vec2f::cast_from(grid_size);

    const OFFSET: Vec2f = vec2(170.0, 0.0);
    const ROTATION_SPEED: f32 = 0.3;
    const APPROACH_CYCLE: f32 = 0.1;

    let time = self.globals.time;
    let angle = f32::consts::TAU * time as f32 * ROTATION_SPEED;
    let offset = OFFSET * (f32::consts::TAU * time as f32 * APPROACH_CYCLE).cos();
    self.metaball_pos = offset.rotated(angle);
  }

  fn get_row(&mut self, first_point_offset: Vec2f, increment_per_point: Vec2f, out: &mut [f32]) {
    for (x, out_value) in out.iter_mut().enumerate() {
      let mut point = vec2n(x as f32) * increment_per_point + first_point_offset;

      let mut value = 0.0_f32;

      point *= self.globals.window_size / self.grid_size_f;

      fn metaball(center: Vec2f, radius: f32, point: Vec2f) -> f32 {
        let distance = point.sqr_distance(center);
        const MIN_SIGNIFICANT_DISTANCE: f32 = 1e-3;
        (2.0 * radius) / distance.max(MIN_SIGNIFICANT_DISTANCE)
      }

      const RADIUS: f32 = 2500.0;

      // value += metaball(vec2n(0.0), RADIUS, point);
      // value += metaball(self.globals.input_state.mouse_pos, RADIUS, point);

      value += metaball(self.metaball_pos, RADIUS, point);
      value += metaball(-self.metaball_pos, RADIUS, point);

      *out_value = value;
    }
  }

  fn get(&mut self, _point: Vec2f) -> f32 { unimplemented!("use get_row") }
}

fn copy_data_from_vec_into_buffer<'obj, Obj: 'obj, T>(
  buffer: &'obj impl BufferBinding<'obj, Obj, T>,
  data_vec: &Vec<T>,
) where
  Obj: Buffer<T>,
  T: Copy,
{
  if buffer.len() != data_vec.capacity() {
    buffer.alloc(data_vec.capacity());
  } else {
    buffer.orphan_data();
  }
  buffer.set_slice(0..data_vec.len(), data_vec);
}
