#version 100

uniform vec2 u_cell_size;
uniform vec2 u_chunk_size;

uniform vec2 u_window_size;
uniform vec2 u_camera_pos;
uniform float u_camera_zoom;

uniform vec2 u_chunk_offset;

attribute vec2 a_pos;

varying vec2 v_pos;

void main() {
  vec2 world_pos = (a_pos * u_chunk_size + u_chunk_offset) * u_cell_size;
  vec2 view_pos = (world_pos - u_camera_pos) * u_camera_zoom / (u_window_size * 0.5);
  gl_Position = vec4(view_pos, 0.0, 1.0);
  v_pos = a_pos;
}
