use cardboard_math::*;
use cardboard_oogl as oogl;
use cardboard_oogl::traits::*;
use prelude_plus::*;

use crate::globals::{Globals, SharedGlobals};

#[derive(Debug)]
pub struct Renderer {
  globals: SharedGlobals,

  vbo: oogl::VertexBuffer<[i8; 2]>,
  white_texture: oogl::Texture2D,

  rectangle_program: oogl::Program,
  rectangle_program_reflection: RendererProgramReflection,
  ellipse_program: oogl::Program,
  ellipse_program_reflection: RendererProgramReflection,
}

impl Renderer {
  pub fn init(globals: SharedGlobals) -> AnyResult<Self> {
    let common_vertex_shader =
      load_shader_asset(&globals, "shaders/shape.vert.glsl", oogl::ShaderType::Vertex)?;

    let rectangle_fragment_shader =
      load_shader_asset(&globals, "shaders/rectangle.frag.glsl", oogl::ShaderType::Fragment)?;

    let ellipse_fragment_shader =
      load_shader_asset(&globals, "shaders/ellipse.frag.glsl", oogl::ShaderType::Fragment)?;

    let rectangle_program = load_program_asset(
      &globals,
      "Renderer.rectangle",
      &[&common_vertex_shader, &rectangle_fragment_shader],
    )?;
    let rectangle_program_reflection = RendererProgramReflection::new(&rectangle_program);

    let ellipse_program = load_program_asset(
      &globals,
      "Renderer.ellipse",
      &[&common_vertex_shader, &ellipse_fragment_shader],
    )?;
    let ellipse_program_reflection = RendererProgramReflection::new(&ellipse_program);

    assert_eq!(
      rectangle_program_reflection.a_pos.location(),
      ellipse_program_reflection.a_pos.location()
    );
    assert_eq!(
      rectangle_program_reflection.a_pos.data_type(),
      ellipse_program_reflection.a_pos.data_type()
    );

    let mut vbo = oogl::VertexBuffer::new(
      globals.gl.share(),
      // this attribute pointer will be the same for both programs because both
      // use the same vertex shader, as such the VBO can be shared
      vec![rectangle_program_reflection.a_pos.to_pointer(oogl::AttribPtrType {
        name: oogl::AttribPtrTypeName::I8,
        len: 2,
        normalize: false,
      })],
    );

    {
      let bound_vbo = vbo.bind();
      bound_vbo.object().set_debug_label(b"vbo");
      bound_vbo.enable_attribs();
      bound_vbo.configure_attribs();
      bound_vbo
        .reserve_and_set(oogl::BufferUsageHint::StaticDraw, &[[-1, -1], [-1, 1], [1, 1], [1, -1]]);
    }

    let texture_unit = oogl::TextureUnit::new(globals.gl.share());

    let mut white_texture =
      oogl::Texture2D::new(globals.gl.share(), oogl::TextureInputFormat::RGBA, None);
    {
      let bound_texture = white_texture.bind(&texture_unit);
      bound_texture.object().set_debug_label(b"white_texture");
      bound_texture.set_wrapping_modes(oogl::TextureWrappingMode::Repeat);
      bound_texture.set_filters(oogl::TextureFilter::Linear, None);
      bound_texture.set_size(vec2n(1));
      bound_texture.reserve_and_set(0, &[0xFF, 0xFF, 0xFF, 0xFF]);
    }

    Ok(Self {
      globals,

      vbo,
      white_texture,

      rectangle_program,
      rectangle_program_reflection,
      ellipse_program,
      ellipse_program_reflection,
    })
  }

  pub fn prepare(&mut self) {
    if self.globals.window_was_resized {
      let window_size = self.globals.window_size;
      for (program, reflection) in &mut [
        (&mut self.rectangle_program, &mut self.rectangle_program_reflection),
        (&mut self.ellipse_program, &mut self.ellipse_program_reflection),
      ] {
        let bound_program = program.bind();
        reflection.u_window_size.set(&bound_program, window_size);
      }
    }
  }

  pub fn draw_shape(&mut self, shape: &mut Shape) {
    let texture_unit = oogl::TextureUnit::new(self.globals.gl.share());
    let (color, _bound_texture) = match &mut shape.fill {
      ShapeFill::Color(color) => (*color, self.white_texture.bind(&texture_unit)),
      ShapeFill::Texture(bound_texture) => (colorn(1.0, 1.0), bound_texture.bind(&texture_unit)),
    };

    let (program, reflection) = match shape.type_ {
      ShapeType::Rectangle => {
        let program = self.rectangle_program.bind();
        (program, &self.rectangle_program_reflection)
      }
      ShapeType::Ellipse => {
        let program = self.ellipse_program.bind();
        (program, &self.ellipse_program_reflection)
      }
    };

    reflection.u_pos.set(&program, shape.pos);
    reflection.u_size.set(&program, shape.size);
    reflection.u_rotation.set(&program, shape.rotation);
    reflection.u_color.set(&program, color);
    reflection.u_tex.set(&program, texture_unit);
    if let Some(clipping) = &shape.fill_clipping {
      reflection.u_tex_clipping_offset.set(&program, clipping.offset);
      reflection.u_tex_clipping_size.set(&program, clipping.size);
    } else {
      reflection.u_tex_clipping_offset.set(&program, vec2n(0.0));
      reflection.u_tex_clipping_size.set(&program, vec2n(1.0));
    }

    let bound_vbo = self.vbo.bind();
    bound_vbo.draw(&program, oogl::DrawPrimitive::TriangleFan);
  }

  pub fn draw_text(&mut self, font: &mut Font, pos: Vec2f, text_block: &mut TextBlock<'_>) {
    let font_char_grid_size_f = Vec2f::cast_from(font.grid_cell_size);
    let font_char_size_f = Vec2f::cast_from(font.character_size);
    let font_texture_size_f = Vec2f::cast_from(font.texture.size());
    let char_size = font_char_size_f * text_block.scale;
    let (text_block_size, char_spacing) = font.measure_size(text_block);

    let mut char_pos = pos;
    char_pos.x += char_spacing.x / 2.0;
    char_pos.x -= text_block_size.x
      * match text_block.horizontal_align {
        TextAlign::Start => 0.0,
        TextAlign::Center => 1.0 / 2.0,
        TextAlign::End => 1.0,
      };
    char_pos.y -= text_block_size.y
      * match text_block.vertical_align {
        TextAlign::Start => 0.5,
        TextAlign::Center => 0.0,
        TextAlign::End => -0.5,
      };

    for chr in text_block.text.chars() {
      let chr = chr as u32;
      if chr >= font.grid_size.x * font.grid_size.y {
        return;
      }

      let char_tex_pos: Vec2f = vec2(chr % font.grid_size.x, chr / font.grid_size.x).cast_into();

      self.draw_shape(&mut Shape {
        type_: ShapeType::Rectangle,
        pos: char_pos,
        size: char_size,
        rotation: 0.0,
        fill: ShapeFill::Texture(&mut font.texture),
        fill_clipping: Some(ShapeClipping {
          offset: (font_char_grid_size_f / font_texture_size_f) * char_tex_pos,
          size: font_char_size_f / font_texture_size_f,
        }),
      });

      char_pos.x += char_spacing.x;
    }
  }
}

oogl::program_reflection_block!({
  #[derive(Debug)]
  struct RendererProgramReflection {
    a_pos: oogl::Attrib<Vec2f>,
    u_window_size: oogl::Uniform<Vec2f>,
    u_pos: oogl::Uniform<Vec2f>,
    u_size: oogl::Uniform<Vec2f>,
    u_rotation: oogl::Uniform<f32>,
    u_color: oogl::Uniform<Colorf>,
    u_tex: oogl::Uniform<oogl::TextureUnit>,
    u_tex_clipping_offset: oogl::Uniform<Vec2f>,
    u_tex_clipping_size: oogl::Uniform<Vec2f>,
  }
});

#[derive(Debug)]
pub struct Shape<'a> {
  pub type_: ShapeType,
  pub pos: Vec2f,
  pub size: Vec2f,
  pub rotation: f32,
  pub fill: ShapeFill<'a>,
  pub fill_clipping: Option<ShapeClipping>,
}

#[derive(Debug)]
pub enum ShapeType {
  Rectangle,
  Ellipse,
}

#[derive(Debug)]
pub enum ShapeFill<'a> {
  Color(Colorf),
  Texture(&'a mut oogl::Texture2D),
}

#[derive(Debug)]
pub struct ShapeClipping {
  pub offset: Vec2f,
  pub size: Vec2f,
}

#[derive(Debug)]
pub struct Font {
  pub texture: oogl::Texture2D,
  pub grid_size: Vec2u32,
  pub grid_cell_size: Vec2u32,
  pub character_size: Vec2u32,
}

impl Font {
  pub fn measure_size(&self, text_block: &TextBlock<'_>) -> (Vec2f, Vec2f) {
    let char_size = Vec2f::cast_from(self.character_size) * text_block.scale;
    let char_spacing = char_size * (vec2n(1.0) + text_block.character_spacing);
    (char_spacing * vec2(text_block.text.len() as f32, 1.0), char_spacing)
  }
}

#[derive(Debug)]
pub struct TextBlock<'a> {
  pub text: &'a str,
  pub scale: Vec2f,
  pub character_spacing: Vec2f,
  pub horizontal_align: TextAlign,
  pub vertical_align: TextAlign,
}

#[derive(Debug, Copy, Clone)]
pub enum TextAlign {
  Start,
  Center,
  End,
}

pub fn load_shader_asset(
  globals: &Globals,
  path: &str,
  type_: oogl::ShaderType,
) -> AnyResult<oogl::Shader> {
  let file_contents = globals.game_fs.read_binary_file(&path)?;
  let shader = compile_shader(globals.gl.share(), &file_contents, type_)
    .with_context(|| format!("Failed to compile the shader '{}'", path))?;
  shader.set_debug_label(path.as_bytes());
  Ok(shader)
}

pub fn load_program_asset(
  globals: &Globals,
  name: &str,
  shaders: &[&oogl::Shader],
) -> AnyResult<oogl::Program> {
  let program = link_program(globals.gl.share(), shaders)
    .with_context(|| format!("Failed to link program '{}'", name))?;
  program.set_debug_label(name.as_bytes());
  program.load_uniform_descriptors();
  Ok(program)
}

pub fn compile_shader(
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

pub fn link_program(
  ctx: oogl::SharedContext,
  shaders: &[&oogl::Shader],
) -> AnyResult<oogl::Program> {
  let program = oogl::Program::new(ctx);
  for shader in shaders {
    program.attach_shader(shader);
  }

  let success = program.link();
  let log = program.get_info_log();
  let log = String::from_utf8_lossy(&log);
  if !success {
    bail!("Program linking error(s):\n{}", log);
  } else if !log.is_empty() {
    warn!("Program linking warning(s):\n{}", log);
  }

  for shader in shaders {
    program.detach_shader(shader);
  }
  Ok(program)
}

pub fn load_texture_asset(
  globals: &Globals,
  path: &str,
  filter: oogl::TextureFilter,
) -> AnyResult<oogl::Texture2D> {
  let file = globals.game_fs.open_file(&path)?;

  let mut texture = load_texture_data_from_png(globals.gl.share(), path.as_bytes(), file)
    .with_context(|| format!("Failed to decode '{}'", path))?;
  let texture_unit = oogl::TextureUnit::new(globals.gl.share());
  let bound_texture = texture.bind(&texture_unit);
  bound_texture.set_wrapping_modes(oogl::TextureWrappingMode::Repeat);
  bound_texture.set_filters(filter, None);
  drop(bound_texture);

  Ok(texture)
}

pub fn load_texture_data_from_png<R: Read>(
  gl: oogl::SharedContext,
  debug_label: &[u8],
  reader: R,
) -> Result<oogl::Texture2D, png::DecodingError> {
  let decoder = png::Decoder::new(reader);
  let (info, mut reader) = decoder.read_info()?;
  let mut buf = vec![0; info.buffer_size()];
  reader.next_frame(&mut buf)?;

  use png::{BitDepth, ColorType};

  match info.bit_depth {
    BitDepth::Eight => {}
    _ => unimplemented!("Unsupported texture bit depth: {:?}", info.bit_depth),
  }

  use oogl::TextureInputFormat as GlFormat;
  let gl_format = match info.color_type {
    ColorType::Grayscale => GlFormat::Luminance,
    ColorType::RGB => GlFormat::RGB,
    ColorType::GrayscaleAlpha => GlFormat::LuminanceAlpha,
    ColorType::RGBA => GlFormat::RGBA,
    _ => unimplemented!("Unsupported texture color type: {:?}", info.color_type),
  };

  let texture_unit = oogl::TextureUnit::new(gl.share());
  let mut texture = oogl::Texture2D::new(gl, gl_format, None);
  let bound_texture = texture.bind(&texture_unit);
  bound_texture.object().set_debug_label(debug_label);
  bound_texture.set_size(vec2(info.width, info.height));
  bound_texture.reserve_and_set(0, &buf);
  drop(bound_texture);

  Ok(texture)
}
