/*!

_**NOTE(2020-11-21)** from my future self:_

Greetings, Seeker! The code you are currently reading is what I consider my
first OpenGL program, as well as the first time I tried out shaders. It was
written sometime in between 2019-10-29, which is presumably when I created the
original Cargo project (and shortly after I had come up with the idea of a
project which would later be named openKrossKod), and 2019-11-21, which is when
I shared a couple of demos I wrote (see below) and stopped adding any code to
this program (unfortunately there wasn't much evidence to narrow down these
dates further). After that it was collecting dust in a directory named
`opengl-playground` for approximately ten months.

The initial name of the Cargo project was in fact `sdl2-min` as it was an
experiment on how few (or many) dependencies would be needed for openKrossKod
(I had grossly underestimated the number); I later used this data in one of my
dependency reduction PRs to the [`sdl2`] crate. Initially I had planned on
using just `SDL_gfx` for graphics, but shortly after I realized that the
capabilities of this library would not be enough, so I made a decision to learn
OpenGL, the general-purpose computer graphics API. After reading a bunch of
OpenGL tutorials (very likely on the evening of 2019-11-21) I took an [example
program](https://github.com/brendanzab/gl-rs/blob/1b698f1e5287cdc4cc5610a9b6e10c9f2905b73c/gl/examples/triangle.rs)
from the [`gl`] crate, adapted it for the SDL library and began writing tiny
demos on top of that. Here is a
[screenshot](https://media.discordapp.net/attachments/293439912362115072/647148408498290698/unknown.png)
of a program which draws a circle around the current position of the mouse
pointer, you can see the fragment shader code in a window in background. Then
there is this
[video](https://cdn.discordapp.com/attachments/382339402338402317/762392858371424276/winter_2019_triangle.mp4)
of a spinning triangle filled with a rainbow-pattern - this is a program the
source code of which you see here (unfortunately I couldn't recover the source
code of the first demo) and to be honest, it is what made me fall in love, so
to speak, with fragment shaders. Another thing I remember writing is a simple
equation plotter - you can see commented-out parts of its code, but I haven't
checked if it works.

After that I haven't touched neither OpenGL nor Rust for a long, long time.
Though I did make a small comeback into computer graphics on 2020-04-09 with my
[shader mod for
CrossCode](https://github.com/dmitmel/crosscode-libshader/tree/bbee14884589fbe44f3bc3996a6a8585caf1b44b)
(demos:
[1](https://cdn.discordapp.com/attachments/276459212807340034/697849408682721360/simplescreenrecorder-2020-04-09_19.43.54.mp4),
[2](https://cdn.discordapp.com/attachments/401253194690199554/697860605742153728/unknown.png),
[3](https://cdn.discordapp.com/attachments/276459212807340034/697929245023928380/unknown.png),
[4](https://cdn.discordapp.com/attachments/683767232668762203/701139440747282542/unknown.png),
[5](https://cdn.discordapp.com/attachments/500980711466074123/698128118619176970/unknown.png)).

And thus I upload this code exactly one year later, in almost untouched state,
to the openKrossKod repository. Granted, I did make a couple of changes: I
rewrote the `Cargo.toml` manifest and fixed an obvious bug with the shader
program linking status not being checked, but other than that there are no
changes! This is literally the program openKrossKod is based on. Well, there
you have it - a great journey of making an opensource CrossCode engine out of
this begins, I guess!

*/

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

use gl::types::*;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;

// const VERTEX_DATA: [GLfloat; 12] =
//   [-1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, 1.0, 1.0];
// const VERTEX_DATA: [GLfloat; 6] = [-0.33, -0.66, 0.33, -0.66, 0.0, 0.66];
const VERTEX_DATA: [GLfloat; 6] = [0.0, 1.0, 0.87, -0.5, -0.5, -0.87];
// const VERTEX_DATA: [GLfloat; 8] = [0.5, 0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5];
// const VERTEX_DATA: [GLfloat; 8] = [1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0];
// const ELEMENT_DATA: [GLuint; 6] = [0, 1, 2, 0, 3, 2];
const ELEMENT_DATA: [GLuint; 3] = [0, 1, 2];

static VS_SRC: &str = include_str!("main.vert");
static FS_SRC: &str = include_str!("main.frag");

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
  unsafe {
    let shader = gl::CreateShader(ty);
    // Attempt to compile the shader
    let c_str = CString::new(src.as_bytes()).unwrap();
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
    gl::CompileShader(shader);

    // Get the compile status
    let mut status = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    // Fail on error
    // if status != (gl::TRUE as GLint) {
    let mut len = 0;
    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
    if len > 0 {
      let mut buf = Vec::with_capacity(len as usize);
      buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
      gl::GetShaderInfoLog(
        shader,
        len,
        ptr::null_mut(),
        buf.as_mut_ptr() as *mut GLchar,
      );
      panic!("{}", str::from_utf8(&buf).expect("ShaderInfoLog not valid utf8"));
    }

    shader
  }
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
  unsafe {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);
    gl::LinkProgram(program);
    // Get the link status
    let mut status = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    // Fail on error
    // if status != (gl::TRUE as GLint) {
    let mut len: GLint = 0;
    gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
    if len > 0 {
      let mut buf = Vec::with_capacity(len as usize);
      buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
      gl::GetProgramInfoLog(
        program,
        len,
        ptr::null_mut(),
        buf.as_mut_ptr() as *mut GLchar,
      );
      panic!(
        "{}",
        str::from_utf8(&buf).expect("ProgramInfoLog not valid utf8")
      );
    }

    program
  }
}

extern "system" fn print_gl_debug_message(
  source: GLenum,
  gltype: GLenum,
  id: GLuint,
  severity: GLenum,
  length: GLsizei,
  message: *const GLchar,
  _user_param: *mut GLvoid,
) {
  let message_bytes = unsafe {
    std::slice::from_raw_parts(message as *const u8, length as usize)
  };
  eprintln!(
    "GL debug: source = 0x{:x}, type = 0x{:x}, id = 0x{:x}, severity = 0x{:x}, message = {:?}",
    source, gltype, id, severity, str::from_utf8(message_bytes)
  );
}

fn main() {
  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();

  let window = video_subsystem
    .window("Window", 800, 600)
    .resizable()
    .opengl()
    .build()
    .unwrap();

  let gl_attr = video_subsystem.gl_attr();
  gl_attr.set_context_profile(GLProfile::Core);
  gl_attr.set_context_version(3, 3);

  let ctx = window.gl_create_context().unwrap();
  gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

  debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
  debug_assert_eq!(gl_attr.context_version(), (3, 3));

  let mut event_pump = sdl_context.event_pump().unwrap();

  // Create GLSL shaders
  let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
  let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
  let program = link_program(vs, fs);

  let mut vao = 0;
  let mut vbo = 0;
  let mut ebo = 0;

  let mut window_size_uniform = 0;

  let mut t = 0.0;
  let mut t_uniform = 0;

  unsafe {
    gl::Enable(gl::DEBUG_OUTPUT);
    gl::DebugMessageCallback(Some(print_gl_debug_message), ptr::null());

    // Create Vertex Array Object
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);

    // Create a Vertex Buffer Object and copy the vertex data to it
    gl::GenBuffers(1, &mut vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl::BufferData(
      gl::ARRAY_BUFFER,
      (VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
      mem::transmute(&VERTEX_DATA[0]),
      gl::STATIC_DRAW,
    );

    gl::GenBuffers(1, &mut ebo);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
    gl::BufferData(
      gl::ELEMENT_ARRAY_BUFFER,
      (ELEMENT_DATA.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
      mem::transmute(&ELEMENT_DATA[0]),
      gl::STATIC_DRAW,
    );

    // Use shader program
    gl::UseProgram(program);
    let c_str = CString::new("out_color").unwrap();
    gl::BindFragDataLocation(program, 0, c_str.as_ptr());

    // Specify the layout of the vertex data
    let c_str = CString::new("position").unwrap();
    let pos_attr = gl::GetAttribLocation(program, c_str.as_ptr());
    gl::EnableVertexAttribArray(pos_attr as GLuint);
    gl::VertexAttribPointer(
      pos_attr as GLuint,
      2,
      gl::FLOAT,
      gl::FALSE,
      0,
      ptr::null(),
    );

    let c_str = CString::new("window_size").unwrap();
    window_size_uniform = gl::GetUniformLocation(program, c_str.as_ptr());

    let window_size = window.size();
    gl::Uniform2f(
      window_size_uniform,
      window_size.0 as GLfloat,
      window_size.1 as GLfloat,
    );

    let c_str = CString::new("t").unwrap();
    t_uniform = gl::GetUniformLocation(program, c_str.as_ptr());
    gl::Uniform1f(t_uniform, t);
  }

  'running: loop {
    unsafe {
      gl::ClearColor(0.3, 0.3, 0.3, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT);

      gl::DrawElements(gl::TRIANGLES, 3, gl::UNSIGNED_INT, ptr::null());

      t += 0.1;
      gl::Uniform1f(t_uniform, t);
    }

    window.gl_swap_window();
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. }
        | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
          break 'running
        }
        Event::Window {
          win_event: sdl2::event::WindowEvent::SizeChanged(width, height),
          ..
        } => unsafe {
          gl::Viewport(0, 0, width, height);
          let window_size = window.size();
          gl::Uniform2f(
            window_size_uniform,
            window_size.0 as GLfloat,
            window_size.1 as GLfloat,
          );
        },
        _ => {}
      }
    }
    std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
  }

  unsafe {
    gl::DeleteProgram(program);
    gl::DeleteShader(fs);
    gl::DeleteShader(vs);
    gl::DeleteBuffers(1, &vbo);
    gl::DeleteVertexArrays(1, &vao);
  }
}
