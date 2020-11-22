#version 100

#ifdef GL_ES
precision highp float;
#endif

uniform vec2 u_window_size;
uniform vec2 u_pos;
uniform vec2 u_size;
uniform float u_rotation;
uniform vec2 u_tex_clipping_offset;
uniform vec2 u_tex_clipping_size;

attribute vec2 a_pos;

varying vec2 v_pos;
varying vec2 v_texcoord;

mat2 rotate(float angle) {
  float s = sin(angle);
  float c = cos(angle);
  return mat2(c, -s, s, c);
}

void main(void) {
  gl_Position = vec4(
    (u_pos + (a_pos * 0.5) * u_size * rotate(u_rotation)) / (u_window_size * 0.5),
    0.0, 1.0
  );
  v_pos = a_pos;
  v_texcoord = vec2(0.5 + 0.5 * a_pos.x, 0.5 - 0.5 * a_pos.y);
  v_texcoord *= u_tex_clipping_size;
  v_texcoord += u_tex_clipping_offset;
}
