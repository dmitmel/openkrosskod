#version 100

uniform vec4 u_global_color;
uniform vec2 u_cell_size;

uniform vec2 u_window_size;
uniform vec2 u_chunk_offset;
uniform vec2 u_camera_pos;
uniform float u_camera_zoom;

attribute vec2 a_pos;
attribute float a_state;

varying vec4 v_color;

const float STATE_COLOR_STEPS = float(32);
const float HOT_HUE = 0.0;
const float COLD_HUE = 2.0 / 3.0;

// <https://github.com/hughsk/glsl-hsv2rgb/blob/1b1112c03408c19c0c64017f433d19f1f11049ba/index.glsl>
vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
  vec2 world_pos = (a_pos + u_chunk_offset) * u_cell_size;
  vec2 view_pos = (world_pos - u_camera_pos) * u_camera_zoom / (u_window_size * 0.5);
  gl_Position = vec4(view_pos, 0.0, 1.0);

  vec3 hsv = vec3(clamp((a_state - 1.0) / STATE_COLOR_STEPS, HOT_HUE, COLD_HUE), 1.0, 1.0);
  v_color = vec4(hsv2rgb(hsv) * float(a_state > 0.0), 1.0) * u_global_color;
}
