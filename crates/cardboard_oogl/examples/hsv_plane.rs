//! Fun fact: This sample was ported from... an Awk program:
//! <https://github.com/dmitmel/dotfiles/blob/9253cb6a08f17bd1a2613bfbbe40975429e94dd6/script-resources/colortest2.awk>

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
}

#[rustfmt::skip]
const VERTEX_DATA: &[Vertex] = &[
  Vertex { pos: vec2( 1.0,  1.0) },
  Vertex { pos: vec2( 1.0, -1.0) },
  Vertex { pos: vec2(-1.0,  1.0) },
  Vertex { pos: vec2(-1.0, -1.0) },
];

static VS_SRC: &str = r#"#version 100
  attribute vec2 a_pos;
  varying   vec2 v_pos;
  void main() {
    gl_Position = vec4(a_pos, 0.0, 1.0);
    v_pos = a_pos;
  }
"#;

static FS_SRC: &str = r#"#version 100
  #ifdef GL_ES
  precision highp float;
  #endif

  const float TAU = 6.28318530717958647692528676655900577;
  const float GRID_SIZE = 24.0;
  const float CIRCLE_RADIUS = 0.8;
  const float CIRCLE_ALPHA_SLOPE = 8.0;

  varying vec2 v_pos;

  // The classic function, taken from <https://stackoverflow.com/a/17897228/12005228>
  vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
  }

  void main() {
    float step = 2.0 / GRID_SIZE;
    vec2 pos = (floor(v_pos / step) + vec2(0.5)) * step;
    if (pos == vec2(0.0)) {
      gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
      return;
    }

    float angle = atan(pos.x, pos.y);
    float dist = length(pos);

    vec3 color = hsv2rgb(vec3(angle / TAU, 1.0, 1.0));
    float alpha = clamp(1.0 + CIRCLE_ALPHA_SLOPE * (CIRCLE_RADIUS - dist), 0.0, 1.0);
    gl_FragColor = vec4(color * alpha, 1.0);
  }
"#;

fn main() {
  let (_sdl_context, _video_subsystem, _sdl_gl_ctx, window, mut event_pump, gl) =
    common::prepare_example_gl_context(common::ExampleConfig {
      name: "hsv_plane",
      window_size: vec2(480, 480),
      ..Default::default()
    });

  let vs = common::compile_shader(gl.share(), VS_SRC, ShaderType::Vertex);
  let fs = common::compile_shader(gl.share(), FS_SRC, ShaderType::Fragment);

  let mut program = common::link_program(gl.share(), &vs, &fs);
  let attr_pos = program.get_attrib::<Vec2f>("a_pos");
  let bound_program = program.bind();

  let mut vbo = VertexBuffer::<Vertex>::new(
    gl.share(),
    BufferUsageHint::StaticDraw,
    vec![attr_pos.to_pointer_simple()],
  );

  let bound_vbo = vbo.bind();
  bound_vbo.enable_attribs();
  bound_vbo.configure_attribs();
  bound_vbo.alloc_and_set(VERTEX_DATA);

  gl.set_clear_color(color(0.0, 0.0, 0.0, 1.0));

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
