// <https://en.wikipedia.org/wiki/Mandelbrot_set>
// <https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set>
// <https://en.wikipedia.org/wiki/Julia_set>

use cardboard_math::*;
use cardboard_oogl as oogl;
use cardboard_oogl::traits::*;
use prelude_plus::*;

use crate::globals::SharedGlobals;
use crate::input::Key;
use crate::renderer;

const CAMERA_ZOOM_SPEED: f32 = 2.4;
const UNIT_SIZE: Vec2f = vec2n(200.0);
const ESCAPE_RADIUS: f32 = 2.0;
const MAX_ITERATIONS: u32 = 50;

#[repr(C, packed)]
#[derive(Copy, Debug, Clone, Default)]
struct Vertex {
  pos: Vec2f,
}

#[derive(Debug)]
pub struct Mandelbrot {
  globals: SharedGlobals,
  vertex_buf: oogl::VertexBuffer<Vertex>,
  program: oogl::Program,
  program_reflection: ProgramReflection,
  camera_pos: Vec2f,
  camera_zoom: f32,
  is_julia_mode: bool,
  starting_point: Vec2f,
}

impl Mandelbrot {
  pub fn init(globals: SharedGlobals) -> AnyResult<Self> {
    use oogl::ShaderType as ShTy;
    let vertex_shader =
      renderer::load_shader_asset(&globals, "shaders/mandelbrot.vert.glsl", ShTy::Vertex)?;
    let fragment_shader =
      renderer::load_shader_asset(&globals, "shaders/mandelbrot.frag.glsl", ShTy::Fragment)?;
    let mut program =
      renderer::load_program_asset(&globals, "Mandelbrot", &[&vertex_shader, &fragment_shader])?;
    let program_reflection = ProgramReflection::new(&program);

    {
      let bound_program = program.bind();
      let reflection = &program_reflection;
      reflection.u_max_iterations.set(&bound_program, &MAX_ITERATIONS);
      reflection.u_escape_radius.set(&bound_program, &ESCAPE_RADIUS);
      reflection.u_unit_size.set(&bound_program, &UNIT_SIZE);
    }

    let mut vertex_buf = oogl::VertexBuffer::new(
      globals.gl.share(),
      oogl::BufferUsageHint::StaticDraw,
      vec![program_reflection.a_pos.to_pointer_simple_with_cast(oogl::AttribPtrTypeName::F32)],
    );
    vertex_buf.set_debug_label(b"Mandelbrot.vertex_buf");
    #[rustfmt::skip]
    vertex_buf.bind().alloc_and_set(&[
      Vertex { pos: vec2( 1.0,  1.0) },
      Vertex { pos: vec2( 1.0, -1.0) },
      Vertex { pos: vec2(-1.0,  1.0) },
      Vertex { pos: vec2(-1.0, -1.0) },
    ]);

    let mut myself = Self {
      globals,
      vertex_buf,
      program,
      program_reflection,
      camera_pos: Vec2f::ZERO,
      camera_zoom: 0.0,
      is_julia_mode: false,
      starting_point: Vec2f::ZERO,
    };
    myself.reset_view();
    Ok(myself)
  }

  fn reset_view(&mut self) {
    self.camera_pos = Vec2f::ZERO;
    self.camera_zoom = 1.0;
  }

  pub fn update(&mut self) {
    let zoom_axis = self.globals.input_state.axis(Key::Minus, Key::Equals) as f32;
    if zoom_axis != 0.0 {
      let zoom_factor = 1.0 + zoom_axis * self.globals.delta_time as f32 * CAMERA_ZOOM_SPEED;
      self.camera_zoom *= zoom_factor;
      let mouse_pos = self.globals.input_state.mouse_pos;
      // <https://stackoverflow.com/a/2919434/12005228>
      self.camera_pos -=
        mouse_pos / (self.camera_zoom * zoom_factor) - mouse_pos / self.camera_zoom;
    }

    if self.globals.input_state.is_key_down(Key::MouseLeft) {
      self.camera_pos -= self.globals.input_state.delta_mouse_pos / self.camera_zoom;
    }

    if self.globals.input_state.is_key_pressed(Key::R) {
      self.reset_view();
      self.starting_point = Vec2f::ZERO;
    }

    if self.globals.input_state.is_key_pressed(Key::J) {
      self.is_julia_mode = !self.is_julia_mode;
    }

    if self.globals.input_state.is_key_down(Key::MouseRight) {
      self.starting_point =
        self.globals.input_state.mouse_pos / self.camera_zoom + self.camera_pos;
    }
  }

  pub fn render(&mut self) {
    let bound_program = self.program.bind();
    let reflection = &self.program_reflection;
    reflection.u_camera_pos.set(&bound_program, &self.camera_pos);
    reflection.u_camera_zoom.set(&bound_program, &self.camera_zoom);
    if self.globals.window_was_resized {
      reflection.u_window_size.set(&bound_program, &self.globals.window_size);
    }
    reflection.u_julia_mode.set(&bound_program, &self.is_julia_mode);
    reflection.u_starting_point.set(&bound_program, &self.starting_point);

    let bound_vertex_buf = self.vertex_buf.bind();
    bound_vertex_buf.enable_attribs();
    bound_vertex_buf.configure_attribs();
    bound_vertex_buf.draw(&bound_program, oogl::DrawPrimitive::TriangleStrip);
  }

  pub fn render_debug_info(
    &mut self,
    renderer: &mut renderer::Renderer,
    font: &mut renderer::Font,
  ) {
    let mut text_block_offset = Vec2f::ZERO;
    for &text in &[
      format!("   pos: {:?} {:?}", self.camera_pos.x, self.camera_pos.y).as_str(),
      format!("  zoom: {:.06e}", self.camera_zoom).as_str(),
      format!(" start: {:?} {:?}", self.starting_point.x, self.starting_point.y).as_str(),
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
    u_max_iterations: oogl::Uniform<u32>,
    u_escape_radius: oogl::Uniform<f32>,
    u_window_size: oogl::Uniform<Vec2f>,
    u_camera_pos: oogl::Uniform<Vec2f>,
    u_camera_zoom: oogl::Uniform<f32>,
    u_unit_size: oogl::Uniform<Vec2f>,
    u_julia_mode: oogl::Uniform<bool>,
    u_starting_point: oogl::Uniform<Vec2f>,
  }
});
