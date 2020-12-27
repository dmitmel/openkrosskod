use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::video::Window;
use std::f32;

use cardboard_math::*;
use cardboard_oogl::*;

#[path = "common.rs"]
mod common;

const CAMERA_POS: Vec3f = vec3(1.2, 1.2, 1.2);
const FOV_ANGLE: f32 = 60.0; // degrees
const NEAR_PLANE: f32 = 1.0;
const FAR_PLANE: f32 = 10.0;
const CLEAR_COLOR: Colorf = colorn(0.2, 1.0);
const ROTATION_SPEED: f32 = 0.005;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Vertex {
  pos: Vec3f,
  texcoord: Vec2f,
}

fn main() {
  let (_sdl_context, _video_subsystem, _sdl_gl_ctx, window, mut event_pump, gl) =
    common::prepare_example_gl_context("textured_cube", vec2(800, 600));

  let vs = common::compile_shader(gl.share(), VS_SRC, ShaderType::Vertex);
  let fs = common::compile_shader(gl.share(), FS_SRC, ShaderType::Fragment);

  let mut program = common::link_program(gl.share(), &vs, &fs);

  let attr_pos = program.get_attrib::<Vec3f>("a_pos");
  let attr_texcoord = program.get_attrib::<Vec2f>("a_texcoord");
  let uni_model_mat = program.get_uniform::<Mat4f>("u_model_mat");
  let uni_view_mat = program.get_uniform::<Mat4f>("u_view_mat");
  let uni_proj_mat = program.get_uniform::<Mat4f>("u_proj_mat");
  let uni_texture = program.get_uniform::<TextureUnit>("u_tex");

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

  const TEXTURE_DATA: &[u8] = include_bytes!("./assets/LearnOpenGL/container.jpeg");
  let mut texture = common::load_jpeg_texture_2d(gl.share(), None, TEXTURE_DATA);
  {
    let bound_texture = texture.bind(None);
    uni_texture.set(&bound_program, &bound_texture.unit());
    bound_texture.set_wrapping_modes(TextureWrappingMode::Repeat);
    bound_texture.set_filters(TextureFilter::Linear, Some(TextureFilter::Linear));
    bound_texture.generate_mipmap();
  }

  uni_view_mat
    .set(&bound_program, &Mat4f::look_to_rh(CAMERA_POS, Vec3f::ZERO - CAMERA_POS, Vec3f::UP));

  let reset_viewport = |gl: &Context, window: &Window| {
    common::reset_gl_viewport(&gl, &window);
    let (w, h) = window.drawable_size();
    let aspect = w as f32 / h as f32;
    uni_proj_mat.set(
      &bound_program,
      &Mat4f::perspective_rh_no(FOV_ANGLE.to_radians(), aspect, NEAR_PLANE, FAR_PLANE),
    );
  };

  reset_viewport(&gl, &window);

  gl.set_clear_color(CLEAR_COLOR);
  gl.set_depth_test_enabled(true);

  let mut rotation = 0.0;

  'running: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } | Event::KeyDown { scancode: Some(Scancode::Escape), .. } => {
          break 'running;
        }

        Event::Window { win_event: WindowEvent::SizeChanged(..), .. } => {
          reset_viewport(&gl, &window);
        }

        _ => {}
      }
    }

    gl.clear(ClearFlags::COLOR | ClearFlags::DEPTH);

    uni_model_mat.set(
      &bound_program,
      &Mat4f::from_axis_angle(Vec3f::UP.normalized(), rotation * f32::consts::TAU),
    );
    rotation = (rotation + ROTATION_SPEED) % 1.0;

    bound_vbo.draw(&bound_program, DrawPrimitive::Triangles);

    window.gl_swap_window();
  }
}

static VS_SRC: &str = r#"#version 100

  uniform mat4 u_model_mat;
  uniform mat4 u_view_mat;
  uniform mat4 u_proj_mat;

  attribute vec3 a_pos;
  attribute vec2 a_texcoord;

  varying vec3 v_pos;
  varying vec2 v_texcoord;

  void main() {
    gl_Position = u_proj_mat * u_view_mat * u_model_mat * vec4(a_pos, 1.0);
    v_texcoord = a_texcoord;
  }
"#;

static FS_SRC: &str = r#"#version 100
  precision highp float;

  uniform sampler2D u_tex;

  varying vec2 v_texcoord;

  void main() {
    gl_FragColor = texture2D(u_tex, v_texcoord);
  }
"#;

#[rustfmt::skip]
const VERTEX_DATA: &[Vertex] = &[
  Vertex { pos: vec3(-0.5, -0.5, -0.5), texcoord: vec2(0.0, 0.0) },
  Vertex { pos: vec3( 0.5, -0.5, -0.5), texcoord: vec2(1.0, 0.0) },
  Vertex { pos: vec3( 0.5,  0.5, -0.5), texcoord: vec2(1.0, 1.0) },
  Vertex { pos: vec3( 0.5,  0.5, -0.5), texcoord: vec2(1.0, 1.0) },
  Vertex { pos: vec3(-0.5,  0.5, -0.5), texcoord: vec2(0.0, 1.0) },
  Vertex { pos: vec3(-0.5, -0.5, -0.5), texcoord: vec2(0.0, 0.0) },

  Vertex { pos: vec3(-0.5, -0.5,  0.5), texcoord: vec2(0.0, 0.0) },
  Vertex { pos: vec3( 0.5, -0.5,  0.5), texcoord: vec2(1.0, 0.0) },
  Vertex { pos: vec3( 0.5,  0.5,  0.5), texcoord: vec2(1.0, 1.0) },
  Vertex { pos: vec3( 0.5,  0.5,  0.5), texcoord: vec2(1.0, 1.0) },
  Vertex { pos: vec3(-0.5,  0.5,  0.5), texcoord: vec2(0.0, 1.0) },
  Vertex { pos: vec3(-0.5, -0.5,  0.5), texcoord: vec2(0.0, 0.0) },

  Vertex { pos: vec3(-0.5,  0.5,  0.5), texcoord: vec2(1.0, 0.0) },
  Vertex { pos: vec3(-0.5,  0.5, -0.5), texcoord: vec2(1.0, 1.0) },
  Vertex { pos: vec3(-0.5, -0.5, -0.5), texcoord: vec2(0.0, 1.0) },
  Vertex { pos: vec3(-0.5, -0.5, -0.5), texcoord: vec2(0.0, 1.0) },
  Vertex { pos: vec3(-0.5, -0.5,  0.5), texcoord: vec2(0.0, 0.0) },
  Vertex { pos: vec3(-0.5,  0.5,  0.5), texcoord: vec2(1.0, 0.0) },

  Vertex { pos: vec3( 0.5,  0.5,  0.5), texcoord: vec2(1.0, 0.0) },
  Vertex { pos: vec3( 0.5,  0.5, -0.5), texcoord: vec2(1.0, 1.0) },
  Vertex { pos: vec3( 0.5, -0.5, -0.5), texcoord: vec2(0.0, 1.0) },
  Vertex { pos: vec3( 0.5, -0.5, -0.5), texcoord: vec2(0.0, 1.0) },
  Vertex { pos: vec3( 0.5, -0.5,  0.5), texcoord: vec2(0.0, 0.0) },
  Vertex { pos: vec3( 0.5,  0.5,  0.5), texcoord: vec2(1.0, 0.0) },

  Vertex { pos: vec3(-0.5, -0.5, -0.5), texcoord: vec2(0.0, 1.0) },
  Vertex { pos: vec3( 0.5, -0.5, -0.5), texcoord: vec2(1.0, 1.0) },
  Vertex { pos: vec3( 0.5, -0.5,  0.5), texcoord: vec2(1.0, 0.0) },
  Vertex { pos: vec3( 0.5, -0.5,  0.5), texcoord: vec2(1.0, 0.0) },
  Vertex { pos: vec3(-0.5, -0.5,  0.5), texcoord: vec2(0.0, 0.0) },
  Vertex { pos: vec3(-0.5, -0.5, -0.5), texcoord: vec2(0.0, 1.0) },

  Vertex { pos: vec3(-0.5,  0.5, -0.5), texcoord: vec2(0.0, 1.0) },
  Vertex { pos: vec3( 0.5,  0.5, -0.5), texcoord: vec2(1.0, 1.0) },
  Vertex { pos: vec3( 0.5,  0.5,  0.5), texcoord: vec2(1.0, 0.0) },
  Vertex { pos: vec3( 0.5,  0.5,  0.5), texcoord: vec2(1.0, 0.0) },
  Vertex { pos: vec3(-0.5,  0.5,  0.5), texcoord: vec2(0.0, 0.0) },
  Vertex { pos: vec3(-0.5,  0.5, -0.5), texcoord: vec2(0.0, 1.0) },
];
