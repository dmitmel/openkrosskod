#version 100

uniform vec2 u_window_size;
uniform vec2 u_camera_pos;
uniform float u_camera_zoom;
uniform vec2 u_unit_size;

attribute vec2 a_pos;

varying vec2 v_screen_pos;
varying vec2 v_world_pos;

void main() {
  gl_Position = vec4(a_pos, 0.0, 1.0);

  v_screen_pos = a_pos;
  v_world_pos = 1.0 / (a_pos * (u_unit_size * 2.0) * u_camera_zoom / u_window_size) + u_camera_pos / u_unit_size;
}
