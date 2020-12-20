use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;

use cardboard_math::*;
use cardboard_oogl::*;

#[path = "common.rs"]
mod common;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Vertex {
  pos: Vec2f,
  texcoord: Vec2f,
}

#[rustfmt::skip]
const VERTEX_DATA: &[Vertex] = &[
  Vertex { pos: vec2( 1.0,  1.0), texcoord: vec2(1.0, 0.0) },
  Vertex { pos: vec2( 1.0, -1.0), texcoord: vec2(1.0, 1.0) },
  Vertex { pos: vec2(-1.0,  1.0), texcoord: vec2(0.0, 0.0) },
  Vertex { pos: vec2(-1.0, -1.0), texcoord: vec2(0.0, 1.0) },
];

static VS_SRC: &str = r#"#version 100
  attribute vec2 a_pos;
  attribute vec2 a_texcoord;
  varying vec2 v_texcoord;
  void main() {
    gl_Position = vec4(a_pos, 0.0, 1.0);
    v_texcoord = a_texcoord;
  }
"#;

static FS_SRC: &str = r#"#version 100
  precision highp float;
  varying vec2 v_texcoord;
  uniform sampler2D u_tex;
  void main() {
    gl_FragColor = texture2D(u_tex, v_texcoord);
  }
"#;

fn main() {
  eprintln!("Try resizing the window");
  let (_sdl_context, _video_subsystem, window, mut event_pump, gl) =
    common::prepare_example_gl_context("manual_mipmap_creation", vec2(800, 600));

  let vs = common::compile_shader(gl.share(), VS_SRC, ShaderType::Vertex);
  let fs = common::compile_shader(gl.share(), FS_SRC, ShaderType::Fragment);

  let mut program = common::link_program(gl.share(), &vs, &fs);

  let attr_pos = program.get_attrib::<Vec2f>("a_pos");
  let attr_texcoord = program.get_attrib::<Vec2f>("a_texcoord");
  let uni_tex = program.get_uniform::<TextureUnit>("u_tex");

  let bound_program = program.bind();

  let mut vbo = VertexBuffer::<Vertex>::new(
    gl.share(),
    BufferUsageHint::StaticDraw,
    vec![attr_pos.to_pointer_simple(), attr_texcoord.to_pointer_simple()],
  );

  let bound_vbo = vbo.bind();
  bound_vbo.enable_attribs();
  bound_vbo.configure_attribs();
  bound_vbo.alloc_and_set(VERTEX_DATA);

  let texture_unit = TextureUnit::new(gl.share());
  uni_tex.set(&bound_program, &texture_unit);

  let mut texture = Texture2D::new(gl.share(), &texture_unit, TextureInputFormat::RGBA, None);
  let bound_texture = texture.bind(&texture_unit);
  bound_texture.set_wrapping_modes(TextureWrappingMode::Repeat);
  bound_texture.set_filters(TextureFilter::Linear, Some(TextureFilter::Linear));

  let texture_size = vec2(1920, 1080);
  bound_texture.set_size(texture_size);

  let generate_texture_data = |color: Color<u8>| -> Vec<u8> {
    let pixels_count = texture_size.x as usize * texture_size.y as usize;
    let mut pixels = vec![0u8; pixels_count * 4];

    let mut current_pixels_slice = &mut pixels[..];
    for _ in 0..pixels_count {
      current_pixels_slice[..4].copy_from_slice(color.as_ref());
      current_pixels_slice = &mut current_pixels_slice[4..];
    }

    pixels
  };

  let texture_data_choices = &[
    generate_texture_data(color(0xff, 0x00, 0x00, 0xff)), // red
    generate_texture_data(color(0x00, 0xff, 0x00, 0xff)), // green
    generate_texture_data(color(0x00, 0x00, 0xff, 0xff)), // blue
  ];

  for lod in 0..bound_texture.object().levels_of_detail_count() {
    let size = bound_texture.object().size_at_level_of_detail(lod);

    let pixels = &texture_data_choices[lod as usize % texture_data_choices.len()];
    bound_texture.alloc_and_set(
      lod,
      // Normally you'd allocate a vector for the downscaled image, run a
      // downscaling algorithm and then feed that into the texture, but because
      // for the sake of the example I'm using single-color images just slicing
      // the pixels vector is fine.
      &pixels[..size.x as usize * size.y as usize * 4],
    );
  }

  gl.set_clear_color(colorn(0.0, 1.0));

  common::reset_gl_viewport(&gl, &window);

  'running: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } | Event::KeyDown { scancode: Some(Scancode::Escape), .. } => {
          break 'running;
        }

        Event::Window { win_event: WindowEvent::SizeChanged(..), .. } => {
          common::reset_gl_viewport(&gl, &window);
        }

        _ => {}
      }
    }

    gl.clear(ClearFlags::COLOR);
    bound_vbo.draw(&bound_program, DrawPrimitive::TriangleStrip);

    window.gl_swap_window();
  }
}
