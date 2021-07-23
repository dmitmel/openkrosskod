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

const CAMERA_ZOOM_SPEED: f64 = 2.4;
const CAMERA_UPDATE_COOLDOWN: f64 = 1.0;
const MAX_PIXELATION_LEVEL: u32 = 4;
const UNIT_SIZE: Vec2f64 = vec2n(200.0);
const ESCAPE_RADIUS: f64 = 2.0;
const MAX_ITERATIONS: u32 = 400;
const DEEP_DIVE_COEFF: f64 = 10.0;

#[rustfmt::skip]
fn iterate(z: Vec2f64, c: Vec2f64) -> Vec2f64 {
  vec2(
    z.x*z.x - z.y*z.y + c.x,
    2.0 * z.x * z.y   + c.y,
  )
}

#[repr(C, packed)]
#[derive(Copy, Debug, Clone, Default)]
struct Vertex {
  pos: Vec2f,
}

#[derive(Debug)]
pub struct Mandelbrot {
  globals: SharedGlobals,
  vertex_buf: oogl::VertexBuffer<Vertex>,
  texture: oogl::Texture2D,
  program: oogl::Program,
  program_reflection: ProgramReflection,

  camera_pos: Vec2f64,
  camera_zoom: f64,
  is_camera_dirty: bool,
  camera_update_timer: f64,
  pixelation_level: u32,

  is_julia_mode: bool,
  starting_point: Vec2f64,
  texture_data: Arc<Vec<u8>>,
  workers: threadpool::ThreadPool,
}

#[derive(Debug)]
struct WorkerChunk {
  pos: Vec2u32,
  size: Vec2u32,
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
      reflection.u_escape_radius.set(&bound_program, &(ESCAPE_RADIUS as f32));
      reflection.u_deep_dive_coeff.set(&bound_program, &(DEEP_DIVE_COEFF as f32));
      reflection.u_unit_size.set(&bound_program, &Vec2f::cast_from(UNIT_SIZE));
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

    let mut tex =
      oogl::Texture2D::new(globals.gl.share(), None, oogl::TextureInputFormat::Luminance, None);
    tex.set_debug_label(b"Mandelbrot.texture");
    {
      let bound_tex = tex.bind(None);
      bound_tex.set_filters(oogl::TextureFilter::Nearest, None);
    }

    let workers = threadpool::Builder::new().thread_name("Mandelbrot.workers".to_owned()).build();

    let mut myself = Self {
      globals,
      vertex_buf,
      texture: tex,
      program,
      program_reflection,

      camera_pos: vec2n(0.0),
      camera_zoom: 0.0,
      is_camera_dirty: false,
      camera_update_timer: 0.0,
      pixelation_level: 0,

      is_julia_mode: false,
      starting_point: vec2n(0.0),
      texture_data: Arc::new(Vec::new()),
      workers,
    };

    myself.reset_view();
    Ok(myself)
  }

  fn reset_view(&mut self) {
    self.camera_pos = vec2n(0.0);
    self.camera_zoom = 1.0;
    self.starting_point = vec2n(0.0);
    self.mark_dirty();
  }

  fn mark_dirty(&mut self) {
    self.is_camera_dirty = true;
    self.camera_update_timer = CAMERA_UPDATE_COOLDOWN;
  }

  pub fn update(&mut self) {
    self.camera_update_timer = (self.camera_update_timer - self.globals.delta_time).max(0.0);

    let mouse_pos = Vec2f64::cast_from(self.globals.input_state.mouse_pos);
    let delta_mouse_pos = Vec2f64::cast_from(self.globals.input_state.delta_mouse_pos);

    let zoom_axis = self.globals.input_state.axis(Key::Minus, Key::Equals);
    if zoom_axis != 0 {
      let zoom_factor = 1.0 + zoom_axis.abs() as f64 * self.globals.delta_time * CAMERA_ZOOM_SPEED;
      let mut new_camera_zoom = self.camera_zoom;
      if zoom_axis > 0 {
        new_camera_zoom *= zoom_factor;
      } else if zoom_axis < 0 {
        new_camera_zoom /= zoom_factor;
      }
      // <https://stackoverflow.com/a/2919434/12005228>
      self.camera_pos -= mouse_pos / new_camera_zoom - mouse_pos / self.camera_zoom;
      self.camera_zoom = new_camera_zoom;
      self.mark_dirty();
    }

    if self.globals.input_state.is_key_down(Key::MouseLeft) {
      self.camera_pos -= delta_mouse_pos / self.camera_zoom;
      self.mark_dirty();
    }

    if self.globals.input_state.is_key_down(Key::MouseRight) {
      self.starting_point = mouse_pos / self.camera_zoom + self.camera_pos;
      self.mark_dirty();
    }

    if self.globals.input_state.is_key_pressed(Key::J) {
      self.is_julia_mode = !self.is_julia_mode;
      self.mark_dirty();
    }

    if self.globals.window_was_resized {
      self.mark_dirty();
    }

    if self.globals.input_state.is_key_pressed(Key::R) {
      self.reset_view();
    }
  }

  pub fn render(&mut self) {
    let bound_program = self.program.bind();
    let reflection = &self.program_reflection;
    reflection.u_camera_pos.set(&bound_program, &Vec2f::cast_from(self.camera_pos));
    reflection.u_camera_zoom.set(&bound_program, &(self.camera_zoom as f32));
    if self.globals.window_was_resized {
      reflection.u_window_size.set(&bound_program, &self.globals.window_size);
    }
    reflection.u_julia_mode.set(&bound_program, &self.is_julia_mode);
    reflection.u_starting_point.set(&bound_program, &Vec2f::cast_from(self.starting_point));

    let bound_tex = self.texture.bind(None);
    reflection.u_tex.set(&bound_program, &bound_tex.unit());

    let target_pixelation_level = (((self.camera_update_timer / CAMERA_UPDATE_COOLDOWN)
      * MAX_PIXELATION_LEVEL as f64) as u32
      + 1)
      .min(MAX_PIXELATION_LEVEL);
    if target_pixelation_level != self.pixelation_level {
      self.is_camera_dirty = true;
    }

    if self.is_camera_dirty {
      let tex_size = self.globals.window_size_i / target_pixelation_level;
      // The need for padding took me an hour to debug and figure out.
      // <https://stackoverflow.com/a/60266711/12005228>
      let tex_row_len =
        oogl::pad_to_alignment(tex_size.x as usize, oogl::TEXTURE_INPUT_DATA_ROW_ALIGN);
      let tex_data_len = tex_row_len * tex_size.y as usize;

      if bound_tex.size() != tex_size {
        bound_tex.set_size(tex_size);
        bound_tex.alloc(0);
        let mut data = Vec::<u8>::with_capacity(tex_data_len);
        unsafe { data.set_len(tex_data_len) };
        self.texture_data = Arc::new(data);
      } else {
        // Orphan the data? Apparently not needed.
        bound_tex.alloc(0);
      }

      // For normalization:
      let inv_tex_size = 1.0 / Vec2f64::cast_from(tex_size);
      // For translation from normalized coords into world coords:
      let norm_to_world_pos_multiplier =
        1.0 / (UNIT_SIZE * self.camera_zoom / Vec2f64::cast_from(self.globals.window_size));
      let camera_world_pos = self.camera_pos / UNIT_SIZE;
      // Other stuff, to please the borrow checker:
      let starting_point = self.starting_point / UNIT_SIZE;
      let is_julia_mode = self.is_julia_mode;

      self.workers.join();

      let mut chunk_start = 0;
      for chunk_len in Self::calculate_chunks(tex_size, self.workers.max_count()) {
        let chunk_end = chunk_start + chunk_len;

        let mut texture_data = Arc::clone(&self.texture_data);
        self.workers.execute(move || {
          for i in chunk_start..chunk_end {
            let (x, y) = (i % tex_size.x as usize, i / tex_size.x as usize);

            let norm_pos = vec2(x as f64 + 0.5, y as f64 + 0.5) * inv_tex_size - 0.5;
            let world_pos = norm_pos * norm_to_world_pos_multiplier + camera_world_pos;

            let mut z = if is_julia_mode { world_pos } else { vec2n(0.0) };
            let c = if is_julia_mode { starting_point } else { world_pos };

            // <https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Periodicity_checking>
            let mut iter = 0;
            let esc_r = ESCAPE_RADIUS as f64;
            while iter < MAX_ITERATIONS && z.sqr_magnitude() <= esc_r * esc_r {
              z = iterate(z, c);
              iter += 1;
            }

            let mut result = 1.0;
            if iter < MAX_ITERATIONS {
              // <https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Continuous_(smooth)_coloring>
              let nu = (z.sqr_magnitude().log2() / 2.0).log2(); // == z.magnitude().log2().log2()
              let smooth_iter = iter as f64 + 1.0 - nu;
              result = smooth_iter / MAX_ITERATIONS as f64;
              if DEEP_DIVE_COEFF != 0.0 {
                // <https://www.math.univ-toulouse.fr/~cheritat/wiki-draw/index.php/Mandelbrot_set#Deep_zooms_and_log-potential_scale>
                // <https://github.com/HackerPoet/FractalSoundExplorer/blob/8855063aa1c8b8ac0a0d61be13ceb85b553698d0/frag.glsl#L122-L124>
                result = (result * f64::consts::PI * DEEP_DIVE_COEFF).cos() * -0.5 + 0.5
              }
            };

            let out = unsafe { Arc::get_mut_unchecked(&mut texture_data) };
            out[x + y * tex_row_len] = (result * u8::MAX as f64) as u8;
          }
        });

        chunk_start = chunk_end;
      }

      self.workers.join();

      bound_tex.set(0, &self.texture_data);
      self.pixelation_level = target_pixelation_level;
      self.is_camera_dirty = false;
    }

    let bound_vertex_buf = self.vertex_buf.bind();
    bound_vertex_buf.enable_attribs();
    bound_vertex_buf.configure_attribs();
    bound_vertex_buf.draw(&bound_program, oogl::DrawPrimitive::TriangleStrip);
  }

  fn calculate_chunks(tex_size: Vec2u32, count: usize) -> Vec<usize> {
    let tex_pixels = tex_size.x as usize * tex_size.y as usize;
    let mut chunks = Vec::with_capacity(count);
    let (quotinent, mut remainder) = (tex_pixels / count, tex_pixels % count);
    for _ in 0..count {
      chunks.push(if remainder > 0 {
        remainder -= 1;
        quotinent + 1
      } else {
        quotinent
      });
    }
    chunks
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
    u_deep_dive_coeff: oogl::Uniform<f32>,
    u_window_size: oogl::Uniform<Vec2f>,
    u_camera_pos: oogl::Uniform<Vec2f>,
    u_camera_zoom: oogl::Uniform<f32>,
    u_unit_size: oogl::Uniform<Vec2f>,
    u_julia_mode: oogl::Uniform<bool>,
    u_starting_point: oogl::Uniform<Vec2f>,
    u_tex: oogl::Uniform<u16>,
  }
});
