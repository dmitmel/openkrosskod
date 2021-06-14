#version 100

uniform vec2 u_grid_size;

attribute vec2 a_pos;
attribute vec2 a_pos_inside_square;
attribute vec4 a_corner_values;

varying vec2 v_pos;
varying vec2 v_pos_inside_square;
varying vec4 v_corner_values;

void main() {
  gl_Position = vec4(a_pos / (u_grid_size * 0.5) - vec2(1.0), 0.0, 1.0);
  // gl_PointSize = 6.0;
  gl_PointSize = 4.0;

  v_pos = a_pos;
  v_corner_values = a_corner_values;
  v_pos_inside_square = a_pos_inside_square;
}
